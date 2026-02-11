#!/usr/bin/env python3
"""Contract lane runner for local fork profile preflight checks."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"
MAX_SECONDS_ENV = "KAMN_KOLME_LOCAL_FORK_PROFILE_PREFLIGHT_CONTRACT_MAX_SECONDS"
DEFAULT_MAX_SECONDS = 45


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local fork profile preflight contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-fork-profile-preflight-summary.json",
        help="Preflight summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-fork-profile-preflight-policy.json",
        help="Preflight policy report output.",
    )
    parser.add_argument(
        "--max-seconds",
        type=int,
        default=None,
        help="Runtime budget value passed through summary metadata.",
    )
    return parser


def parse_max_seconds(raw_value: str) -> int:
    if not raw_value.isdigit() or int(raw_value) <= 0:
        raise ValueError("max-seconds must be a positive integer")
    return int(raw_value)


def main() -> int:
    args = build_parser().parse_args()

    try:
        max_seconds = (
            args.max_seconds
            if args.max_seconds is not None
            else parse_max_seconds(os.environ.get(MAX_SECONDS_ENV, str(DEFAULT_MAX_SECONDS)).strip())
        )
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    if max_seconds <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected local fork profile preflight runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local fork profile preflight policy checker to be executable", file=sys.stderr)
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
            "/tmp/kolme_fork",
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

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    readme_text = README_FILE.read_text(encoding="utf-8")
    if "run_local_kolme_fork_profile_preflight_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local fork profile preflight runner", file=sys.stderr)
        return 1
    if "check_local_kolme_fork_profile_preflight_policy.py" not in doc_text:
        print(
            "expected Kolme devnet ops doc to reference local fork profile preflight policy checker",
            file=sys.stderr,
        )
        return 1
    if "run_local_kolme_fork_profile_preflight_contract_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local fork profile preflight contract lane", file=sys.stderr)
        return 1
    if "Regression: #1697" not in doc_text:
        print(
            "expected Kolme devnet ops doc to include local fork profile preflight contract-lane regression marker",
            file=sys.stderr,
        )
        return 1
    if "check_local_kolme_fork_profile_preflight_policy.py" not in readme_text:
        print("expected README to reference local fork profile preflight policy checker", file=sys.stderr)
        return 1
    if "run_local_kolme_fork_profile_preflight_contract_lane.sh" not in readme_text:
        print("expected README to reference local fork profile preflight contract lane", file=sys.stderr)
        return 1

    print("local fork profile preflight contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
