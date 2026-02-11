#!/usr/bin/env python3
"""Contract lane runner for local bootstrap health-check checks."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_bootstrap_health_checks.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_bootstrap_health_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local bootstrap health-check contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-bootstrap-summary.json",
        help="Local bootstrap summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-bootstrap-policy.json",
        help="Local bootstrap policy report output.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected local bootstrap health-check runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local bootstrap health policy checker to be executable", file=sys.stderr)
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
    if "run_local_bootstrap_health_checks.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local bootstrap runner", file=sys.stderr)
        return 1
    if "check_local_bootstrap_health_policy.py" not in doc_text:
        print("expected Kolme devnet ops doc to reference local bootstrap policy checker", file=sys.stderr)
        return 1
    if "run_local_bootstrap_health_checks_contract_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local bootstrap contract lane", file=sys.stderr)
        return 1
    if "Regression: #1692" not in doc_text:
        print("expected Kolme devnet ops doc to include local bootstrap policy regression marker", file=sys.stderr)
        return 1
    if "check_local_bootstrap_health_policy.py" not in readme_text:
        print("expected README to reference local bootstrap policy checker", file=sys.stderr)
        return 1
    if "run_local_bootstrap_health_checks_contract_lane.sh" not in readme_text:
        print("expected README to reference local bootstrap contract lane", file=sys.stderr)
        return 1

    print("local bootstrap health-check contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
