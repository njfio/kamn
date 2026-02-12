#!/usr/bin/env python3
"""Contract lane runner for local runtime-commit submit/finality evidence checks."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_runtime_commit_live_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_runtime_commit_live_evidence_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
FOUNDATION_DOC = ROOT_DIR / "docs/foundation/kolme-runtime-commit-client.md"
CI_STRATEGY_DOC = ROOT_DIR / "docs/ci/strategy.md"
README_FILE = ROOT_DIR / "README.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local runtime-commit live finality evidence contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-runtime-commit-live-summary.json",
        help="Runtime-commit live summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-runtime-commit-live-policy.json",
        help="Policy checker report output.",
    )
    parser.add_argument(
        "--live-output-file",
        default="/tmp/kolme-local-runtime-commit-live-output.txt",
        help="Runtime-commit live command output capture path.",
    )
    parser.add_argument(
        "--finality-output-file",
        default="/tmp/kolme-local-runtime-commit-live-finality-output.txt",
        help="Runtime-commit finality command output capture path.",
    )
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_KOLME_LOCAL_RUNTIME_COMMIT_LIVE_FINALITY_MAX_SECONDS", "120"),
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--finality-max-seconds",
        default="15",
        help="Finality command runtime budget in seconds.",
    )
    return parser


def _is_positive_integer(raw_value: str) -> bool:
    return raw_value.isdigit() and int(raw_value) > 0


def main() -> int:
    args = build_parser().parse_args()

    if not _is_positive_integer(args.max_seconds):
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    if not _is_positive_integer(args.finality_max_seconds):
        print("finality-max-seconds must be a positive integer", file=sys.stderr)
        return 1

    max_seconds = int(args.max_seconds)
    finality_max_seconds = int(args.finality_max_seconds)

    for path in (RUNNER, CHECKER):
        if not path.is_file() or not path.stat().st_mode & 0o111:
            print(f"expected executable dependency: {path}", file=sys.stderr)
            return 1

    for path in (DOC_FILE, FOUNDATION_DOC, CI_STRATEGY_DOC, README_FILE):
        if not path.is_file():
            print(f"expected documentation file to exist: {path}", file=sys.stderr)
            return 1

    required_doc_markers = (
        "run_local_runtime_commit_live_finality_evidence_contract_lane.sh",
        "check_local_runtime_commit_live_evidence_policy.py",
        "submit_evidence_marker_present",
        "finality_evidence_marker_present",
        "Regression: #2099",
    )
    for marker in required_doc_markers:
        for doc_path in (DOC_FILE, FOUNDATION_DOC, CI_STRATEGY_DOC):
            doc_text = doc_path.read_text(encoding="utf-8")
            if marker not in doc_text:
                print(f"expected documentation marker '{marker}' in {doc_path}", file=sys.stderr)
                return 1

    if "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" not in README_FILE.read_text(
        encoding="utf-8"
    ):
        print(
            "expected README to reference local runtime-commit finality evidence contract lane",
            file=sys.stderr,
        )
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
            "--live-output-file",
            args.live_output_file,
            "--finality-output-file",
            args.finality_output_file,
            "--max-seconds",
            str(max_seconds),
            "--finality-max-seconds",
            str(finality_max_seconds),
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
            "--skip-preflight",
            "--live-command",
            "printf 'status=submitted\\nintegration_kolme_fork_live_node_submit_reaches_endpoint\\n'",
            "--finality-command",
            "printf 'finality=final\\n'",
            "--max-seconds",
            str(max_seconds),
            "--finality-max-seconds",
            str(finality_max_seconds),
            "--output-json",
            args.output_json,
            "--live-output-file",
            args.live_output_file,
            "--finality-output-file",
            args.finality_output_file,
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
            "live_runtime_commit_and_finality_commands_passed",
            "--output-json",
            args.policy_output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    summary_payload = json_load(Path(args.output_json))
    if summary_payload.get("schema_version") != "kamn.kolme.local-runtime-commit-live-summary.v1":
        print("unexpected runtime-commit live summary schema", file=sys.stderr)
        return 1
    if summary_payload.get("submit_evidence_marker_present") is not True:
        print("expected submit_evidence_marker_present=true", file=sys.stderr)
        return 1
    if summary_payload.get("finality_evidence_marker_present") is not True:
        print("expected finality_evidence_marker_present=true", file=sys.stderr)
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"runtime-commit live finality evidence contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local runtime-commit live finality evidence contract lane tests passed.")
    return 0


def json_load(path: Path) -> dict[str, object]:
    import json

    return json.loads(path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    raise SystemExit(main())
