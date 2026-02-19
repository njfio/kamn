#!/usr/bin/env python3
"""Contract lane runner for triadic devnet smoke checks."""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
SMOKE_RUNNER = ROOT_DIR / "scripts/kolme/run_triadic_devnet_smoke.sh"
VALIDATOR = ROOT_DIR / "scripts/kolme/validate_triadic_devnet_smoke.py"
POLICY_CHECKER = ROOT_DIR / "scripts/kolme/check_triadic_devnet_smoke_policy.py"
FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_compatibility/devnet_smoke_markers.json"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE = ROOT_DIR / "docs/ci/strategy.md"
README_FILE = ROOT_DIR / "README.md"
MAX_SECONDS = 180


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run triadic devnet smoke contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default=str(ROOT_DIR / "triadic-devnet-smoke-report.json"),
        help="Triadic devnet smoke report output path.",
    )
    parser.add_argument(
        "--policy-output-json",
        default=str(ROOT_DIR / "triadic-devnet-smoke-policy-report.json"),
        help="Triadic devnet smoke policy report output path.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()

    if not SMOKE_RUNNER.is_file() or not SMOKE_RUNNER.stat().st_mode & 0o111:
        print("expected triadic devnet smoke runner to be executable", file=sys.stderr)
        return 1
    if not VALIDATOR.is_file() or not VALIDATOR.stat().st_mode & 0o111:
        print("expected triadic devnet smoke validator to be executable", file=sys.stderr)
        return 1
    if not POLICY_CHECKER.is_file() or not POLICY_CHECKER.stat().st_mode & 0o111:
        print("expected triadic devnet smoke policy checker to be executable", file=sys.stderr)
        return 1
    if not FIXTURE_FILE.is_file():
        print("expected triadic devnet smoke marker fixture file to exist", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file() or not CI_DOC_FILE.is_file() or not README_FILE.is_file():
        print("expected triadic devnet docs to exist", file=sys.stderr)
        return 1

    output_json = Path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    policy_output_json = Path(args.policy_output_json)
    policy_output_json.parent.mkdir(parents=True, exist_ok=True)

    start_epoch = time.monotonic()

    with tempfile.NamedTemporaryFile(prefix="triadic-devnet-markers-", delete=True) as temp_markers:
        subprocess.run(
            [
                "bash",
                str(SMOKE_RUNNER),
                "--output-file",
                temp_markers.name,
                "--max-seconds",
                str(MAX_SECONDS),
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

        subprocess.run(
            [
                "python3",
                str(VALIDATOR),
                "--fixture",
                str(FIXTURE_FILE),
                "--marker-file",
                temp_markers.name,
                "--output-json",
                str(output_json),
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

        subprocess.run(
            [
                "python3",
                str(POLICY_CHECKER),
                "--report-file",
                str(output_json),
                "--expected-final-decision",
                "GO",
                "--require-reason-code",
                "triadic_devnet_smoke_policy_passed",
                "--output-json",
                str(policy_output_json),
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    ci_doc_text = CI_DOC_FILE.read_text(encoding="utf-8")
    readme_text = README_FILE.read_text(encoding="utf-8")
    if "run_triadic_devnet_smoke.sh" not in doc_text:
        print("expected devnet ops doc to reference triadic devnet smoke runner command", file=sys.stderr)
        return 1
    if "validate_triadic_devnet_smoke.py" not in doc_text:
        print("expected devnet ops doc to reference triadic devnet smoke validator command", file=sys.stderr)
        return 1
    if "check_triadic_devnet_smoke_policy.py" not in doc_text:
        print("expected devnet ops doc to reference triadic devnet smoke policy checker command", file=sys.stderr)
        return 1
    if "check_triadic_devnet_smoke_policy.py" not in ci_doc_text:
        print("expected CI strategy doc to reference triadic devnet smoke policy checker command", file=sys.stderr)
        return 1
    if "run_triadic_devnet_smoke.sh" not in readme_text:
        print("expected README to reference triadic devnet smoke command", file=sys.stderr)
        return 1
    if "check_triadic_devnet_smoke_policy.py" not in readme_text:
        print("expected README to reference triadic devnet smoke policy checker command", file=sys.stderr)
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > MAX_SECONDS:
        print(f"triadic devnet smoke contract lane exceeded runtime budget: {elapsed_seconds}s", file=sys.stderr)
        return 1

    print("triadic devnet smoke contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
