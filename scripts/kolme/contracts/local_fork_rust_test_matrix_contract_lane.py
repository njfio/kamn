#!/usr/bin/env python3
"""Contract lane runner for local Kolme fork rust matrix checks."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_rust_test_matrix_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
MAX_SECONDS_ENV = "KAMN_KOLME_LOCAL_FORK_RUST_MATRIX_MAX_SECONDS"
DEFAULT_MAX_SECONDS = 120


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local fork rust test matrix contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-fork-rust-test-matrix-summary.json",
        help="Matrix summary output path.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-fork-rust-test-matrix-policy.json",
        help="Policy report output path.",
    )
    parser.add_argument(
        "--checkout-path",
        default="/tmp/kolme_fork",
        help="Local fork checkout path used for run-mode metadata checks.",
    )
    return parser


def parse_max_seconds() -> int:
    raw_value = os.environ.get(MAX_SECONDS_ENV, str(DEFAULT_MAX_SECONDS)).strip()
    if not raw_value.isdigit() or int(raw_value) <= 0:
        raise ValueError(f"{MAX_SECONDS_ENV} must be a positive integer")
    return int(raw_value)


def main() -> int:
    args = build_parser().parse_args()

    try:
        max_seconds = parse_max_seconds()
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected local fork rust test matrix lane runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local fork rust test matrix policy checker to be executable", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    subprocess.run(
        [
            "bash",
            str(RUNNER),
            "--mode",
            "dry-run",
            "--checkout-path",
            args.checkout_path,
            "--max-seconds",
            str(max_seconds),
            "--output-json",
            args.output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    subprocess.run(
        [
            "python3",
            str(CHECKER),
            "--report-file",
            args.output_json,
            "--expected-final-decision",
            "GO",
            "--ci-fast-gate",
            "PASS",
            "--require-reason-code",
            "dry_run_no_commands_executed",
            "--output-json",
            args.policy_output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    run_env = os.environ.copy()
    run_env["KAMN_KOLME_LOCAL_HEAVY"] = "1"
    subprocess.run(
        [
            "bash",
            str(RUNNER),
            "--mode",
            "run",
            "--checkout-path",
            args.checkout_path,
            "--matrix-command",
            "printf 'matrix_contract_ok_1\\n'",
            "--matrix-command",
            "printf 'matrix_contract_ok_2\\n'",
            "--max-seconds",
            str(max_seconds),
            "--output-json",
            args.output_json,
        ],
        cwd=ROOT_DIR,
        env=run_env,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    subprocess.run(
        [
            "python3",
            str(CHECKER),
            "--report-file",
            args.output_json,
            "--expected-final-decision",
            "GO",
            "--ci-fast-gate",
            "PASS",
            "--require-reason-code",
            "fork_rust_test_matrix_passed",
            "--output-json",
            args.policy_output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    if "run_local_kolme_fork_rust_test_matrix_lane.sh" not in doc_text:
        print(
            "expected Kolme devnet ops doc to reference local fork rust test matrix lane runner",
            file=sys.stderr,
        )
        return 1
    if "check_local_kolme_fork_rust_test_matrix_policy.py" not in doc_text:
        print(
            "expected Kolme devnet ops doc to reference local fork rust test matrix policy checker",
            file=sys.stderr,
        )
        return 1
    if "run_local_kolme_fork_rust_test_matrix_contract_lane.sh" not in doc_text:
        print(
            "expected Kolme devnet ops doc to reference local fork rust test matrix contract lane",
            file=sys.stderr,
        )
        return 1
    if "Regression: #1541" not in doc_text:
        print(
            "expected Kolme devnet ops doc to include local fork rust test matrix regression marker",
            file=sys.stderr,
        )
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"local fork rust test matrix contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local fork rust test matrix contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
