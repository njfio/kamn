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
CI_DOC_FILE = ROOT_DIR / "docs/ci/strategy.md"
README_FILE = ROOT_DIR / "README.md"
MAX_SECONDS_ENV = "KAMN_KOLME_LOCAL_FORK_RUST_MATRIX_MAX_SECONDS"
DEFAULT_MAX_SECONDS = 120
EVIDENCE_BUNDLE_SCHEMA_MARKER = (
    "evidence_bundle_schema_version=kamn.kolme.local-fork-rust-test-matrix-evidence-bundle.v1"
)
# Regression: #1541
# Regression: #2329
REQUIRED_DOC_MARKERS = [
    "run_local_kolme_fork_rust_test_matrix_lane.sh",
    "check_local_kolme_fork_rust_test_matrix_policy.py",
    "run_local_kolme_fork_rust_test_matrix_contract_lane.sh",
    EVIDENCE_BUNDLE_SCHEMA_MARKER,
    "evidence_bundle",
    "Regression: #1541",
    "Regression: #2329",
]


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
    docs_files = [DOC_FILE, CI_DOC_FILE, README_FILE]
    for docs_file in docs_files:
        if not docs_file.is_file():
            print(f"expected docs parity file to exist: {docs_file}", file=sys.stderr)
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

    for docs_file in docs_files:
        doc_text = docs_file.read_text(encoding="utf-8")
        for marker in REQUIRED_DOC_MARKERS:
            if marker not in doc_text:
                print(
                    f"expected docs parity marker '{marker}' in {docs_file}",
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
