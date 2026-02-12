#!/usr/bin/env python3
"""Contract lane runner for local KAMN live runtime integration checks."""

from __future__ import annotations

import argparse
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kamn_live_runtime_integration_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local KAMN live runtime integration contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
        help="Runtime integration summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-kamn-live-runtime-integration-policy.json",
        help="Policy checker report output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="210",
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--fork-chain-version",
        default="v0.15.2",
        help="Required fork-info chain_version query value.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()

    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    max_seconds = int(args.max_seconds)

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected local KAMN live runtime integration runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local KAMN live runtime integration policy checker to be executable", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1
    if not README_FILE.is_file():
        print("expected README to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    with tempfile.TemporaryDirectory(prefix="kolme-live-runtime-integration-") as temp_dir:
        temp_path = Path(temp_dir)
        checkout_path = temp_path / "kolme_fork"
        runtime_commit_output_file = temp_path / "runtime_commit_endpoint.log"
        checkout_path.mkdir(parents=True, exist_ok=True)

        subprocess.run(["git", "-C", str(checkout_path), "init", "-q"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "checkout", "-q", "-b", "main"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.email", "ci@example.com"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.name", "CI Runner"], check=True)
        (checkout_path / "README.md").write_text(
            "local KAMN live runtime integration fixture\n",
            encoding="utf-8",
        )
        subprocess.run(["git", "-C", str(checkout_path), "add", "README.md"], check=True)
        subprocess.run(
            [
                "git",
                "-C",
                str(checkout_path),
                "commit",
                "-q",
                "-m",
                "init runtime integration fixture",
            ],
            check=True,
        )
        subprocess.run(
            [
                "git",
                "-C",
                str(checkout_path),
                "remote",
                "add",
                "origin",
                "https://github.com/njfio/kolme_fork.git",
            ],
            check=True,
        )

        subprocess.run(
            [
                "bash",
                str(RUNNER),
                "--mode",
                "dry-run",
                "--checkout-path",
                str(checkout_path),
                "--expected-remote-url",
                "https://github.com/njfio/kolme_fork.git",
                "--expected-ref",
                "refs/heads/main",
                "--base-url",
                "http://127.0.0.1:3000",
                "--fork-chain-version",
                args.fork_chain_version,
                "--max-seconds",
                str(max_seconds),
                "--localhost-signed-max-seconds",
                "45",
                "--runtime-commit-output-file",
                str(runtime_commit_output_file),
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
    if "run_local_kamn_live_runtime_integration_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local KAMN live runtime integration runner", file=sys.stderr)
        return 1
    if "check_local_kamn_live_runtime_integration_policy.py" not in doc_text:
        print("expected Kolme devnet ops doc to reference local KAMN live runtime integration policy checker", file=sys.stderr)
        return 1
    if "run_local_kamn_live_runtime_integration_contract_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local KAMN live runtime integration contract lane", file=sys.stderr)
        return 1
    # Regression: #1971
    if "--runtime-commit-finality-command" not in doc_text:
        print("expected Kolme devnet ops doc to document runtime finality pass-through command option", file=sys.stderr)
        return 1
    if "run_localhost_signed_integration_contract_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference localhost signed integration prerequisite lane", file=sys.stderr)
        return 1
    if "Regression: #1489" not in doc_text:
        print("expected Kolme devnet ops doc to include local KAMN live runtime integration regression marker", file=sys.stderr)
        return 1
    if "Regression: #1971" not in doc_text:
        print("expected Kolme devnet ops doc to include runtime finality pass-through regression marker", file=sys.stderr)
        return 1
    if "run_local_kamn_live_runtime_integration_contract_lane.sh" not in readme_text:
        print("expected README to reference local KAMN live runtime integration contract lane", file=sys.stderr)
        return 1
    if "--runtime-commit-finality-command" not in readme_text:
        print("expected README to document runtime finality pass-through command option", file=sys.stderr)
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"local KAMN live runtime integration contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local KAMN live runtime integration contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
