#!/usr/bin/env python3
"""Contract lane runner for local native API parity live-proof checks."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_native_api_parity_live_proof_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_native_api_parity_live_proof_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local native API parity live-proof contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-native-api-parity-live-proof-summary.json",
        help="Native parity summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-native-api-parity-live-proof-policy.json",
        help="Policy checker report output.",
    )
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_KOLME_LOCAL_NATIVE_API_PARITY_MAX_SECONDS", "180"),
        help="Total runtime budget in seconds.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()
    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    max_seconds = int(args.max_seconds)

    for script in (RUNNER, CHECKER):
        if not script.is_file() or not script.stat().st_mode & 0o111:
            print(f"expected executable dependency: {script}", file=sys.stderr)
            return 1

    if not DOC_FILE.is_file() or not README_FILE.is_file():
        print("expected docs to exist", file=sys.stderr)
        return 1

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    readme_text = README_FILE.read_text(encoding="utf-8")
    for marker in (
        "run_local_native_api_parity_live_proof_lane.sh",
        "check_local_native_api_parity_live_proof_policy.py",
        "run_local_native_api_parity_live_proof_contract_lane.sh",
        "Regression: #1465",
    ):
        if marker not in doc_text:
            print(f"expected Kolme devnet ops doc marker: {marker}", file=sys.stderr)
            return 1
    if "run_local_native_api_parity_live_proof_contract_lane.sh" not in readme_text:
        print("expected README to reference local native parity contract lane", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    subprocess.run(
        [
            "bash",
            str(RUNNER),
            "--mode",
            "dry-run",
            "--output-json",
            args.output_json,
            "--max-seconds",
            str(max_seconds),
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

    run_env = dict(os.environ)
    run_env["KAMN_KOLME_LOCAL_HEAVY"] = "1"
    subprocess.run(
        [
            "bash",
            str(RUNNER),
            "--mode",
            "run",
            "--nonce-command",
            "printf 'nonce_ok\\n'",
            "--broadcast-command",
            "printf 'broadcast_ok\\n'",
            "--finality-command",
            "printf 'finality_ok\\n'",
            "--max-seconds",
            str(max_seconds),
            "--output-json",
            args.output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        env=run_env,
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
            "native_parity_live_proof_passed",
            "--output-json",
            args.policy_output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"local native API parity live proof contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local native API parity live proof contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
