#!/usr/bin/env python3
"""Contract lane runner for local fork self-test checks."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_self_test_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_self_test_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"
MAX_SECONDS_ENV = "KAMN_KOLME_LOCAL_FORK_SELF_TEST_CONTRACT_MAX_SECONDS"
MATRIX_MAX_SECONDS_ENV = "KAMN_KOLME_LOCAL_FORK_SELF_TEST_CONTRACT_MATRIX_MAX_SECONDS"
MATRIX_CARGO_PROFILE_ENV = "KAMN_KOLME_LOCAL_FORK_SELF_TEST_CONTRACT_CARGO_PROFILE"
DEFAULT_MAX_SECONDS = 120
DEFAULT_MATRIX_MAX_SECONDS = 60
DEFAULT_MATRIX_CARGO_PROFILE = "portable"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local fork self-test contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-fork-self-test-summary.json",
        help="Self-test summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-fork-self-test-policy.json",
        help="Self-test policy report output.",
    )
    parser.add_argument(
        "--checkout-path",
        default="/tmp/kolme_fork",
        help="Local fork checkout path passed to dry-run runner.",
    )
    parser.add_argument(
        "--expected-remote-url",
        default="https://github.com/njfio/kolme_fork.git",
        help="Expected checkout remote URL passed to dry-run runner.",
    )
    parser.add_argument(
        "--expected-ref",
        default="refs/heads/main",
        help="Expected checkout ref passed to dry-run runner.",
    )
    parser.add_argument(
        "--max-seconds",
        type=int,
        default=None,
        help="Runtime budget value passed through summary metadata.",
    )
    parser.add_argument(
        "--matrix-max-seconds",
        type=int,
        default=None,
        help="Matrix runtime budget passed through summary metadata.",
    )
    parser.add_argument(
        "--matrix-cargo-profile",
        default=None,
        help="Matrix cargo profile passed through summary metadata.",
    )
    return parser


def parse_positive_int(raw_value: str) -> int:
    if not raw_value.isdigit() or int(raw_value) <= 0:
        raise ValueError("max-second arguments must be positive integers")
    return int(raw_value)


def main() -> int:
    args = build_parser().parse_args()

    try:
        max_seconds = (
            args.max_seconds
            if args.max_seconds is not None
            else parse_positive_int(os.environ.get(MAX_SECONDS_ENV, str(DEFAULT_MAX_SECONDS)).strip())
        )
        matrix_max_seconds = (
            args.matrix_max_seconds
            if args.matrix_max_seconds is not None
            else parse_positive_int(
                os.environ.get(MATRIX_MAX_SECONDS_ENV, str(DEFAULT_MATRIX_MAX_SECONDS)).strip()
            )
        )
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    if max_seconds <= 0 or matrix_max_seconds <= 0:
        print("max-second arguments must be positive integers", file=sys.stderr)
        return 1

    matrix_cargo_profile = (
        args.matrix_cargo_profile
        if args.matrix_cargo_profile is not None
        else os.environ.get(MATRIX_CARGO_PROFILE_ENV, DEFAULT_MATRIX_CARGO_PROFILE).strip()
    )
    if matrix_cargo_profile not in {"strict", "portable"}:
        print("matrix-cargo-profile must be one of: strict, portable", file=sys.stderr)
        return 1

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected local fork self-test runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local fork self-test policy checker to be executable", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1
    if not README_FILE.is_file():
        print("expected README to exist", file=sys.stderr)
        return 1

    subprocess.run(
        [
            "bash",
            str(RUNNER),
            "--mode",
            "dry-run",
            "--checkout-path",
            args.checkout_path,
            "--expected-remote-url",
            args.expected_remote_url,
            "--expected-ref",
            args.expected_ref,
            "--max-seconds",
            str(max_seconds),
            "--matrix-max-seconds",
            str(matrix_max_seconds),
            "--matrix-cargo-profile",
            matrix_cargo_profile,
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

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    readme_text = README_FILE.read_text(encoding="utf-8")
    if "run_local_kolme_fork_self_test_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local fork self-test runner", file=sys.stderr)
        return 1
    if "check_local_kolme_fork_self_test_policy.py" not in doc_text:
        print("expected Kolme devnet ops doc to reference local fork self-test policy checker", file=sys.stderr)
        return 1
    if "run_local_kolme_fork_self_test_contract_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local fork self-test contract lane", file=sys.stderr)
        return 1
    if "Regression: #1702" not in doc_text:
        print(
            "expected Kolme devnet ops doc to include local fork self-test contract-lane regression marker",
            file=sys.stderr,
        )
        return 1
    if "check_local_kolme_fork_self_test_policy.py" not in readme_text:
        print("expected README to reference local fork self-test policy checker", file=sys.stderr)
        return 1
    if "run_local_kolme_fork_self_test_contract_lane.sh" not in readme_text:
        print("expected README to reference local fork self-test contract lane", file=sys.stderr)
        return 1

    print("local fork self-test contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
