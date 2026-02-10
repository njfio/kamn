#!/usr/bin/env python3
"""SDK example fixture drift contract-lane runner."""

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
        description="Run SDK example fixture drift contract lane."
    )
    parser.add_argument("--output-report")
    return parser


def require_positive_int(raw_value: str) -> int:
    """Require a positive runtime budget integer."""
    try:
        value = int(raw_value)
    except (TypeError, ValueError):
        fail("KAMN_SDK_EXAMPLE_FIXTURE_DRIFT_MAX_SECONDS must be a positive integer")
    if value <= 0:
        fail("KAMN_SDK_EXAMPLE_FIXTURE_DRIFT_MAX_SECONDS must be a positive integer")
    return value


def run_command(command: list[str]) -> tuple[int, str]:
    """Run command in repository root and capture stdout."""
    completed = subprocess.run(
        command,
        cwd=ROOT_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return completed.returncode, completed.stdout or ""


def require_output_line(output: str, marker: str, error_message: str) -> None:
    """Require exact marker line in output."""
    if marker not in output.splitlines():
        fail(error_message)


def main(argv: list[str]) -> int:
    args = build_parser().parse_args(argv)

    checker = ROOT_DIR / "scripts/sdk/check_example_fixture_drift.py"
    policy_checker = ROOT_DIR / "scripts/sdk/check_example_fixture_drift_policy.sh"
    fixture = ROOT_DIR / "fixtures/sdk_parity/register_validation_cases.json"
    snapshot = ROOT_DIR / "fixtures/sdk_parity/register_validation_snapshot.json"
    planning_doc = ROOT_DIR / "docs/planning/sdk-parity-wave.md"
    rust_doc = ROOT_DIR / "docs/foundation/rust-sdk-alpha.md"
    python_doc = ROOT_DIR / "docs/foundation/python-sdk-beta.md"
    typescript_doc = ROOT_DIR / "docs/foundation/typescript-sdk-beta.md"

    for required_exec in (checker, policy_checker):
        if not required_exec.is_file() or not os.access(required_exec, os.X_OK):
            fail(f"expected executable script '{required_exec}'")

    for required_file in (
        fixture,
        snapshot,
        planning_doc,
        rust_doc,
        python_doc,
        typescript_doc,
    ):
        if not required_file.is_file():
            fail(f"expected required file '{required_file}'")

    max_seconds = require_positive_int(
        os.getenv("KAMN_SDK_EXAMPLE_FIXTURE_DRIFT_MAX_SECONDS", "45")
    )

    start_epoch = int(time.time())
    with tempfile.TemporaryDirectory() as tmp_dir:
        output_report = (
            Path(args.output_report)
            if args.output_report is not None and args.output_report != ""
            else Path(tmp_dir) / "sdk-example-fixture-drift-report.json"
        )

        checker_rc, checker_output = run_command(
            [
                "python3",
                str(checker),
                "--fixture",
                str(fixture),
                "--snapshot",
                str(snapshot),
                "--output-json",
                str(output_report),
            ]
        )
        if checker_rc != 0:
            fail("expected sdk example fixture drift checker to pass in contract lane")
        require_output_line(
            checker_output,
            "status=pass",
            "expected sdk example fixture drift checker to pass in contract lane",
        )
        require_output_line(
            checker_output,
            "reason_codes=none",
            "expected sdk example fixture drift checker reason codes to be none in contract lane",
        )

        policy_rc, policy_output = run_command(
            ["bash", str(policy_checker), "--report-file", str(output_report)]
        )
        if policy_rc != 0:
            fail("expected sdk example fixture drift policy checker status marker")
        require_output_line(
            policy_output,
            "status=ok",
            "expected sdk example fixture drift policy checker status marker",
        )
        require_output_line(
            policy_output,
            "final_decision=GO",
            "expected sdk example fixture drift policy checker final decision to be GO",
        )

        for doc in (planning_doc, rust_doc, python_doc, typescript_doc):
            text = doc.read_text(encoding="utf-8")
            if "run_example_fixture_drift_contract_lane.sh" not in text:
                fail(
                    "expected documentation "
                    f"'{doc}' to reference sdk example fixture drift contract lane command"
                )
            if "register_validation_snapshot.json" not in text:
                fail(
                    "expected documentation "
                    f"'{doc}' to reference sdk fixture snapshot path"
                )

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_seconds:
            fail(
                "sdk example fixture drift contract lane exceeded runtime budget: "
                f"{elapsed_seconds}s"
            )

        print("status=ok")
        print(f"report_file={output_report}")
        print("final_decision=GO")
        print("reason_key=sdk_example_fixture_drift_reason_codes:GO:v1")
        print("sdk example fixture drift contract lane tests passed.")

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
