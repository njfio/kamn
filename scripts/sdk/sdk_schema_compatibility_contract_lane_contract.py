#!/usr/bin/env python3
"""SDK schema compatibility contract-lane runner."""

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
        description="Run SDK schema compatibility contract lane."
    )
    parser.add_argument("--output-file")
    parser.add_argument("--lane", default="contract")
    return parser


def require_positive_int(raw_value: str) -> int:
    """Require a positive integer runtime budget with shell-compatible error."""
    try:
        value = int(raw_value)
    except (TypeError, ValueError):
        fail("KAMN_SDK_SCHEMA_COMPATIBILITY_MAX_SECONDS must be a positive integer")
    if value <= 0:
        fail("KAMN_SDK_SCHEMA_COMPATIBILITY_MAX_SECONDS must be a positive integer")
    return value


def run_command(command: list[str]) -> tuple[int, str]:
    """Run command in repo root and capture stdout."""
    completed = subprocess.run(
        command,
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stdout or ""


def require_output_line(output: str, marker: str, message: str) -> None:
    """Require an exact line in command output."""
    if marker not in output.splitlines():
        fail(message)


def main(argv: list[str]) -> int:
    args = build_parser().parse_args(argv)

    lane = args.lane
    if lane not in {"contract", "deep"}:
        fail("--lane must be contract or deep")

    matrix_runner = ROOT_DIR / "scripts/sdk/run_sdk_parity_matrix.sh"
    generator = ROOT_DIR / "scripts/sdk/generate_sdk_schema_compatibility_evidence_bundle.sh"
    policy_checker = ROOT_DIR / "scripts/sdk/check_sdk_schema_compatibility_policy.sh"
    fixture = ROOT_DIR / "fixtures/sdk_parity/register_validation_cases.json"

    if not matrix_runner.is_file() or not os.access(matrix_runner, os.X_OK):
        fail("expected sdk parity matrix runner to be executable")
    if not generator.is_file() or not os.access(generator, os.X_OK):
        fail("expected sdk schema compatibility evidence generator to be executable")
    if not policy_checker.is_file() or not os.access(policy_checker, os.X_OK):
        fail("expected sdk schema compatibility policy checker to be executable")
    if not fixture.is_file():
        fail(f"sdk parity fixture not found: {fixture}")

    max_seconds = require_positive_int(
        os.getenv("KAMN_SDK_SCHEMA_COMPATIBILITY_MAX_SECONDS", "60")
    )
    start_epoch = int(time.time())

    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp_path = Path(tmp_dir)
        output_file = (
            Path(args.output_file)
            if args.output_file is not None and args.output_file != ""
            else tmp_path / "sdk-schema-compatibility-contract.json"
        )
        matrix_report = tmp_path / "sdk-parity-matrix-report.json"

        matrix_rc, matrix_output = run_command(
            [
                "bash",
                str(matrix_runner),
                "--fixture",
                str(fixture),
                "--output-json",
                str(matrix_report),
            ]
        )
        if matrix_rc != 0:
            fail("expected sdk parity matrix contract run to pass")
        if "status=pass" not in matrix_output:
            fail("expected sdk parity matrix contract run to pass")

        generation_rc, generation_output = run_command(
            [
                "bash",
                str(generator),
                "--output-file",
                str(output_file),
                "--lane",
                lane,
                "--matrix-report-file",
                str(matrix_report),
                "--compatibility-suite-status",
                "pass",
                "--runtime-budget-status",
                "within",
                "--ci-fast-gate",
                "PASS",
            ]
        )
        if generation_rc != 0:
            fail("expected sdk schema compatibility bundle decision to be GO")
        require_output_line(
            generation_output,
            "final_decision=GO",
            "expected sdk schema compatibility bundle decision to be GO",
        )
        require_output_line(
            generation_output,
            "reason_key=sdk_schema_compatibility_reason_codes:GO:v1",
            "expected sdk schema compatibility GO reason key",
        )

        policy_rc, policy_output = run_command(
            ["bash", str(policy_checker), "--bundle-file", str(output_file)]
        )
        if policy_rc != 0:
            fail("expected sdk schema compatibility policy status marker")
        require_output_line(
            policy_output,
            "status=ok",
            "expected sdk schema compatibility policy status marker",
        )
        require_output_line(
            policy_output,
            "final_decision=GO",
            "expected sdk schema compatibility policy decision to be GO",
        )
        require_output_line(
            policy_output,
            "failed_checks=none",
            "expected sdk schema compatibility policy failed checks to be none",
        )

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_seconds:
            fail(
                "sdk schema compatibility contract lane exceeded runtime budget: "
                f"{elapsed_seconds}s"
            )

        print("status=ok")
        print(f"lane={lane}")
        print(f"bundle_file={output_file}")
        print(f"matrix_report={matrix_report}")
        print("final_decision=GO")
        print("sdk schema compatibility contract lane tests passed.")

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
