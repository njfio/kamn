#!/usr/bin/env python3
"""Contract lane runner for local KAMN live runtime real-node profile checks."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE = ROOT_DIR / "docs/ci/strategy.md"
README_FILE = ROOT_DIR / "README.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local KAMN live runtime real-node profile contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
        help="Runtime integration summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-kamn-live-runtime-real-node-policy.json",
        help="Real-node profile policy checker output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="180",
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--fork-chain-version",
        default="v0.15.2",
        help="Required fork-info chain_version query value.",
    )
    return parser


def ensure_markers_present(text: str, markers: list[str], source_name: str) -> list[str]:
    missing: list[str] = []
    for marker in markers:
        if marker not in text:
            missing.append(f"{source_name}_missing_marker:{marker}")
    return missing


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
        print("expected local KAMN live runtime real-node profile policy checker to be executable", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1
    if not CI_DOC_FILE.is_file():
        print("expected CI strategy documentation to exist", file=sys.stderr)
        return 1
    if not README_FILE.is_file():
        print("expected README to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    with tempfile.TemporaryDirectory(prefix="kolme-runtime-real-node-contract-") as temp_dir:
        temp_path = Path(temp_dir)
        checkout_path = temp_path / "kolme_fork"
        runtime_commit_live_summary = temp_path / "runtime_commit_live_summary.json"
        runtime_commit_live_policy = temp_path / "runtime_commit_live_policy.json"
        checkout_path.mkdir(parents=True, exist_ok=True)

        subprocess.run(["git", "-C", str(checkout_path), "init", "-q"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "checkout", "-q", "-b", "main"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.email", "ci@example.com"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.name", "CI Runner"], check=True)
        (checkout_path / "README.md").write_text(
            "local KAMN real-node profile contract fixture\n",
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
                "init runtime real-node profile fixture",
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
                "--runtime-profile",
                "real-node",
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
                "--runtime-provider-client-contract",
                "KolmeRuntimeCommitLiveProvider",
                "--runtime-commit-live-summary",
                str(runtime_commit_live_summary),
                "--runtime-commit-live-policy-report",
                str(runtime_commit_live_policy),
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
                "--require-non-synthetic-run-evidence",
                "--output-json",
                args.policy_output_json,
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

    summary = json.loads(Path(args.output_json).read_text(encoding="utf-8"))
    policy = json.loads(Path(args.policy_output_json).read_text(encoding="utf-8"))
    if summary.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
        print("unexpected runtime integration summary schema for real-node profile contract lane", file=sys.stderr)
        return 1
    if summary.get("status") != "ok":
        print("expected runtime integration summary status ok for real-node profile contract lane", file=sys.stderr)
        return 1
    if summary.get("reason_code") != "dry_run_no_commands_executed":
        print("expected dry-run reason code for real-node profile contract lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_profile") != "real-node":
        print("expected runtime_profile=real-node in contract-lane summary", file=sys.stderr)
        return 1
    runtime_commit_command = summary.get("runtime_commit_command")
    if not isinstance(runtime_commit_command, str):
        print("expected runtime_commit_command in contract-lane summary", file=sys.stderr)
        return 1
    if "--require-non-synthetic-run-evidence" not in runtime_commit_command:
        print("expected strict non-synthetic runtime marker in contract-lane summary command", file=sys.stderr)
        return 1
    contracts = summary.get("contracts", {})
    if not isinstance(contracts, dict):
        print("expected contracts object in real-node profile contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_profile") != "real-node":
        print("expected contracts.runtime_profile=real-node in contract-lane summary", file=sys.stderr)
        return 1
    if policy.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-real-node-policy-report.v1":
        print("unexpected real-node profile policy schema in contract-lane output", file=sys.stderr)
        return 1
    if policy.get("final_decision") != "GO":
        print("expected real-node profile policy final_decision GO", file=sys.stderr)
        return 1

    doc_markers = [
        "--runtime-profile real-node",
        "check_local_kamn_live_runtime_real_node_profile_policy.py",
        "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh",
        "--require-non-synthetic-run-evidence",
        "Regression: #2139",
    ]
    ci_doc_markers = [
        "--runtime-profile real-node",
        "check_local_kamn_live_runtime_real_node_profile_policy.py",
        "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh",
        "--require-non-synthetic-run-evidence",
        "Regression: #2139",
    ]
    readme_markers = [
        "--runtime-profile real-node",
        "check_local_kamn_live_runtime_real_node_profile_policy.py",
        "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh",
        "--require-non-synthetic-run-evidence",
        "Regression: #2139",
    ]

    missing_markers: list[str] = []
    missing_markers.extend(
        ensure_markers_present(
            DOC_FILE.read_text(encoding="utf-8"), doc_markers, "docs/planning/kolme-devnet-ops.md"
        )
    )
    missing_markers.extend(
        ensure_markers_present(
            CI_DOC_FILE.read_text(encoding="utf-8"), ci_doc_markers, "docs/ci/strategy.md"
        )
    )
    missing_markers.extend(
        ensure_markers_present(
            README_FILE.read_text(encoding="utf-8"), readme_markers, "README.md"
        )
    )
    if missing_markers:
        print(",".join(missing_markers), file=sys.stderr)
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"local KAMN live runtime real-node profile contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local KAMN live runtime real-node profile contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
