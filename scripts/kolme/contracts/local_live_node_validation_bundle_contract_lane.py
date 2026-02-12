#!/usr/bin/env python3
"""Contract lane runner for local live-node validation bundle checks."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_live_node_validation_bundle_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_live_node_validation_bundle_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE = ROOT_DIR / "docs/ci/strategy.md"
README_FILE = ROOT_DIR / "README.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local live-node validation bundle contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-live-node-validation-bundle-summary.json",
        help="Bundle summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-live-node-validation-bundle-policy.json",
        help="Bundle policy checker report output.",
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
        print("expected local live-node validation bundle runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local live-node validation bundle policy checker to be executable", file=sys.stderr)
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

    with tempfile.TemporaryDirectory(prefix="kolme-live-node-bundle-contract-") as temp_dir:
        temp_path = Path(temp_dir)
        checkout_path = temp_path / "kolme_fork"
        checkout_path.mkdir(parents=True, exist_ok=True)

        subprocess.run(["git", "-C", str(checkout_path), "init", "-q"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "checkout", "-q", "-b", "main"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.email", "ci@example.com"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.name", "CI Runner"], check=True)
        (checkout_path / "README.md").write_text(
            "local live-node validation bundle fixture\n",
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
                "init live-node bundle fixture",
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

    summary = json.loads(Path(args.output_json).read_text(encoding="utf-8"))
    policy = json.loads(Path(args.policy_output_json).read_text(encoding="utf-8"))
    if summary.get("schema_version") != "kamn.kolme.local-live-node-validation-bundle-summary.v1":
        print("unexpected local live-node validation bundle summary schema", file=sys.stderr)
        return 1
    if summary.get("status") != "ok":
        print("expected local live-node validation bundle summary status ok", file=sys.stderr)
        return 1
    if summary.get("reason_code") != "dry_run_no_commands_executed":
        print("expected dry-run reason code in local live-node validation bundle summary", file=sys.stderr)
        return 1
    if summary.get("ci_fast_gate_eligible") is not False:
        print("expected local-only fast-gate exclusion marker in bundle summary", file=sys.stderr)
        return 1
    contracts = summary.get("contracts", {})
    if not isinstance(contracts, dict):
        print("expected bundle summary contracts object", file=sys.stderr)
        return 1
    if contracts.get("ci_fast_gate_scope") != "local-only":
        print("expected local-only fast-gate scope contract marker in bundle summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
        print("expected runtime provider contract marker in bundle summary", file=sys.stderr)
        return 1
    if policy.get("schema_version") != "kamn.kolme.local-live-node-validation-bundle-policy-report.v1":
        print("unexpected local live-node validation bundle policy report schema", file=sys.stderr)
        return 1
    if policy.get("final_decision") != "GO":
        print("expected bundle policy final_decision GO", file=sys.stderr)
        return 1

    doc_markers = [
        "run_local_live_node_validation_bundle_lane.sh",
        "check_local_live_node_validation_bundle_policy.py",
        "run_local_live_node_validation_bundle_contract_lane.sh",
        "Regression: #2134",
    ]
    ci_doc_markers = [
        "run_local_live_node_validation_bundle_lane.sh",
        "check_local_live_node_validation_bundle_policy.py",
        "run_local_live_node_validation_bundle_contract_lane.sh",
        "Regression: #2134",
    ]
    readme_markers = [
        "run_local_live_node_validation_bundle_lane.sh",
        "check_local_live_node_validation_bundle_policy.py",
        "run_local_live_node_validation_bundle_contract_lane.sh",
        "Regression: #2134",
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
            f"local live-node validation bundle contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local live-node validation bundle contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
