#!/usr/bin/env python3
"""Live transport smoke parity contract lane runner."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail  # noqa: E402


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run SDK live transport smoke parity contract lane."
    )
    parser.add_argument("--output-file")
    return parser


def require_positive_budget(raw_value: str) -> int:
    """Require positive integer runtime budget with stable shell-compatible errors."""
    try:
        value = int(raw_value)
    except (TypeError, ValueError):
        fail("KAMN_SDK_SMOKE_PARITY_CONTRACT_MAX_SECONDS must be a positive integer")
    if value <= 0:
        fail("KAMN_SDK_SMOKE_PARITY_CONTRACT_MAX_SECONDS must be a positive integer")
    return value


def ensure_contains_line(output: str, expected: str, error_message: str) -> None:
    """Require an exact output line."""
    if expected not in output.splitlines():
        fail(error_message)


def ensure_contains_text(output: str, expected: str, error_message: str) -> None:
    """Require output to contain expected substring."""
    if expected not in output:
        fail(error_message)


def run_command(
    command: list[str],
    *,
    env: dict[str, str] | None = None,
    capture_stderr: bool = False,
) -> tuple[int, str]:
    """Run command and return (return_code, captured_output)."""
    merged_env = os.environ.copy()
    if env is not None:
        merged_env.update(env)

    completed = subprocess.run(
        command,
        cwd=ROOT_DIR,
        env=merged_env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT if capture_stderr else subprocess.PIPE,
        text=True,
        check=False,
    )
    if capture_stderr:
        output = completed.stdout or ""
    else:
        output = completed.stdout or ""
    return completed.returncode, output


def assert_doc_contains(doc_text: str, marker: str, error_message: str) -> None:
    """Require contract marker to remain documented."""
    if marker not in doc_text:
        fail(error_message)


def main(argv: list[str]) -> int:
    args = build_parser().parse_args(argv)

    smoke_runner = ROOT_DIR / "scripts/sdk/run_live_transport_smoke_parity_lane.sh"
    policy_checker = ROOT_DIR / "scripts/sdk/check_live_transport_smoke_parity_policy.sh"
    rust_sdk_doc = ROOT_DIR / "docs/foundation/rust-sdk-alpha.md"

    if not smoke_runner.is_file() or not os.access(smoke_runner, os.X_OK):
        fail("expected sdk live transport smoke parity runner to be executable")
    if not policy_checker.is_file() or not os.access(policy_checker, os.X_OK):
        fail("expected sdk live transport smoke parity policy checker to be executable")
    if not rust_sdk_doc.is_file():
        fail("expected rust sdk alpha doc to exist")

    max_contract_seconds = require_positive_budget(
        os.getenv("KAMN_SDK_SMOKE_PARITY_CONTRACT_MAX_SECONDS", "240")
    )

    start_epoch = int(time.time())
    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp_path = Path(tmp_dir)
        output_file = (
            Path(args.output_file)
            if args.output_file
            else tmp_path / "sdk-live-transport-smoke-go.json"
        )

        go_rc, go_output = run_command(
            [
                "bash",
                str(smoke_runner),
                "--output-json",
                str(output_file),
            ],
            env={"KAMN_SDK_SMOKE_PARITY_MAX_SECONDS": str(max_contract_seconds)},
        )
        if go_rc != 0:
            fail("expected sdk live transport smoke parity lane to report pass status")
        ensure_contains_line(
            go_output,
            "status=pass",
            "expected sdk live transport smoke parity lane to report pass status",
        )
        ensure_contains_line(
            go_output,
            "final_decision=GO",
            "expected sdk live transport smoke parity lane to report GO decision",
        )

        go_policy_rc, go_policy_output = run_command(
            ["bash", str(policy_checker), "--report-file", str(output_file)]
        )
        if go_policy_rc != 0:
            fail("expected sdk live transport smoke parity policy check decision to be GO")
        ensure_contains_line(
            go_policy_output,
            "final_decision=GO",
            "expected sdk live transport smoke parity policy check decision to be GO",
        )
        ensure_contains_line(
            go_policy_output,
            "failed_checks=none",
            "expected sdk live transport smoke parity GO policy check to have no failed checks",
        )

        runtime_budget_report = (
            tmp_path / "sdk-live-transport-smoke-runtime-budget-no-go.json"
        )
        runtime_rc, runtime_output = run_command(
            [
                "bash",
                str(smoke_runner),
                "--output-json",
                str(runtime_budget_report),
            ],
            env={
                "KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS": "true",
                "KAMN_SDK_SMOKE_PARITY_FAKE_DELAY_SECONDS": "1",
                "KAMN_SDK_SMOKE_PARITY_MAX_SECONDS": "0",
            },
            capture_stderr=True,
        )
        if runtime_rc == 0:
            fail("expected runtime-budget failure run to fail closed")
        ensure_contains_text(
            runtime_output,
            "runtime_budget_exceeded",
            "expected runtime-budget failure run to emit runtime_budget_exceeded",
        )

        runtime_policy_rc, runtime_policy_output = run_command(
            ["bash", str(policy_checker), "--report-file", str(runtime_budget_report)]
        )
        if runtime_policy_rc != 0:
            fail("expected runtime-budget policy check to return NO-GO")
        ensure_contains_line(
            runtime_policy_output,
            "final_decision=NO-GO",
            "expected runtime-budget policy check to return NO-GO",
        )
        ensure_contains_text(
            runtime_policy_output,
            "runtime_budget_exceeded",
            "expected runtime-budget policy check failed checks to include runtime_budget_exceeded",
        )

        retry_budget_report = tmp_path / "sdk-live-transport-smoke-retry-budget-no-go.json"
        retry_rc, retry_output = run_command(
            [
                "bash",
                str(smoke_runner),
                "--output-json",
                str(retry_budget_report),
            ],
            env={
                "KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS": "true",
                "KAMN_SDK_SMOKE_PARITY_FORCE_FAILURE": "true",
                "KAMN_SDK_SMOKE_PARITY_MAX_RETRIES": "1",
            },
            capture_stderr=True,
        )
        if retry_rc == 0:
            fail("expected retry-budget failure run to fail closed")
        ensure_contains_text(
            retry_output,
            "retry_budget_exceeded",
            "expected retry-budget failure run to emit retry_budget_exceeded",
        )

        retry_policy_rc, retry_policy_output = run_command(
            ["bash", str(policy_checker), "--report-file", str(retry_budget_report)]
        )
        if retry_policy_rc != 0:
            fail("expected retry-budget policy check to return NO-GO")
        ensure_contains_line(
            retry_policy_output,
            "final_decision=NO-GO",
            "expected retry-budget policy check to return NO-GO",
        )
        ensure_contains_text(
            retry_policy_output,
            "retry_budget_exceeded",
            "expected retry-budget policy check failed checks to include retry_budget_exceeded",
        )

        doc_text = rust_sdk_doc.read_text(encoding="utf-8")
        assert_doc_contains(
            doc_text,
            "run_live_transport_smoke_parity_lane.sh",
            "expected rust sdk alpha doc to reference sdk smoke parity lane runner",
        )
        assert_doc_contains(
            doc_text,
            "check_live_transport_smoke_parity_policy.sh",
            "expected rust sdk alpha doc to reference sdk smoke parity policy checker",
        )
        assert_doc_contains(
            doc_text,
            "run_live_transport_smoke_parity_contract_lane.sh",
            "expected rust sdk alpha doc to reference sdk smoke parity contract lane runner",
        )
        assert_doc_contains(
            doc_text,
            "kamn.sdk.live-transport-smoke-parity-report.v1",
            "expected rust sdk alpha doc to reference sdk smoke parity report schema marker",
        )
        assert_doc_contains(
            doc_text,
            "KAMN_SDK_SMOKE_PARITY_MAX_SECONDS",
            "expected rust sdk alpha doc to reference sdk smoke parity runtime budget marker",
        )
        assert_doc_contains(
            doc_text,
            "KAMN_SDK_SMOKE_PARITY_MAX_RETRIES",
            "expected rust sdk alpha doc to reference sdk smoke parity retry budget marker",
        )
        assert_doc_contains(
            doc_text,
            "Regression: #938",
            "expected rust sdk alpha doc to include Regression: #938 marker",
        )

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_contract_seconds:
            fail(
                "sdk live transport smoke parity contract lane exceeded runtime budget: "
                f"{elapsed_seconds}s"
            )

        print("status=ok")
        print(f"bundle_file={output_file}")
        print("final_decision=GO")
        print("sdk live transport smoke parity contract lane tests passed.")

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
