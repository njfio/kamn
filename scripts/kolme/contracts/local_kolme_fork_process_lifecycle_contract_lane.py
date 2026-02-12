#!/usr/bin/env python3
"""Contract lane runner for local Kolme fork process lifecycle checks."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local Kolme fork process lifecycle contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-fork-process-lifecycle-summary.json",
        help="Process lifecycle summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-fork-process-lifecycle-policy.json",
        help="Policy checker report output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="300",
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
        print("expected local Kolme fork process lifecycle runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local Kolme fork process lifecycle policy checker to be executable", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1
    if not README_FILE.is_file():
        print("expected README to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    with tempfile.TemporaryDirectory(prefix="kolme-process-lifecycle-") as temp_dir:
        temp_path = Path(temp_dir)
        checkout_path = temp_path / "kolme_fork"
        process_output_file = temp_path / "kolme_process.log"
        integration_report = temp_path / "kolme_runtime_integration_summary.json"
        runtime_policy_report = temp_path / "kolme_runtime_commit_live_policy.json"
        rollback_evidence_file = temp_path / "kolme_process_lifecycle_rollback_evidence.json"
        recovery_evidence_file = temp_path / "kolme_process_lifecycle_recovery_evidence.json"
        checkout_path.mkdir(parents=True, exist_ok=True)

        subprocess.run(["git", "-C", str(checkout_path), "init", "-q"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "checkout", "-q", "-b", "main"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.email", "ci@example.com"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.name", "CI Runner"], check=True)
        (checkout_path / "README.md").write_text(
            "local Kolme fork process lifecycle fixture\n",
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
                "init process lifecycle fixture",
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
                "--process-output-file",
                str(process_output_file),
                "--integration-report",
                str(integration_report),
                "--integration-runtime-commit-live-policy-report",
                str(runtime_policy_report),
                "--rollback-evidence-file",
                str(rollback_evidence_file),
                "--recovery-evidence-file",
                str(recovery_evidence_file),
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
        if summary.get("integration_runtime_commit_live_policy_report") != str(runtime_policy_report):
            print(
                "expected process lifecycle summary to expose integration runtime policy report path",
                file=sys.stderr,
            )
            return 1
        if str(runtime_policy_report) not in summary.get("artifact_paths", []):
            print(
                "expected process lifecycle summary artifact paths to include integration runtime policy report",
                file=sys.stderr,
            )
            return 1
        if summary.get("rollback_evidence_file") != str(rollback_evidence_file):
            print(
                "expected process lifecycle summary to expose rollback evidence file path",
                file=sys.stderr,
            )
            return 1
        if summary.get("recovery_evidence_file") != str(recovery_evidence_file):
            print(
                "expected process lifecycle summary to expose recovery evidence file path",
                file=sys.stderr,
            )
            return 1
        if summary.get("rollback_evidence_status") != "planned":
            print(
                "expected process lifecycle summary to keep rollback evidence status planned in dry-run",
                file=sys.stderr,
            )
            return 1
        if summary.get("recovery_evidence_status") != "planned":
            print(
                "expected process lifecycle summary to keep recovery evidence status planned in dry-run",
                file=sys.stderr,
            )
            return 1
        if str(rollback_evidence_file) not in summary.get("artifact_paths", []):
            print(
                "expected process lifecycle summary artifact paths to include rollback evidence file",
                file=sys.stderr,
            )
            return 1
        if str(recovery_evidence_file) not in summary.get("artifact_paths", []):
            print(
                "expected process lifecycle summary artifact paths to include recovery evidence file",
                file=sys.stderr,
            )
            return 1
        runtime_policy_reason_code = summary.get("integration_runtime_commit_policy_reason_code")
        if not isinstance(runtime_policy_reason_code, str) or not runtime_policy_reason_code.strip():
            print(
                "expected process lifecycle summary to expose integration runtime policy reason code",
                file=sys.stderr,
            )
            return 1

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    readme_text = README_FILE.read_text(encoding="utf-8")
    if "run_local_kolme_fork_process_lifecycle_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local Kolme fork process lifecycle runner", file=sys.stderr)
        return 1
    if "check_local_kolme_fork_process_lifecycle_policy.py" not in doc_text:
        print("expected Kolme devnet ops doc to reference local Kolme fork process lifecycle policy checker", file=sys.stderr)
        return 1
    if "run_local_kolme_fork_process_lifecycle_contract_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local Kolme fork process lifecycle contract lane", file=sys.stderr)
        return 1
    # Regression: #1973
    if "--integration-runtime-commit-finality-command" not in doc_text:
        print(
            "expected Kolme devnet ops doc to document process lifecycle integration finality pass-through option",
            file=sys.stderr,
        )
        return 1
    # Regression: #2104
    if "--integration-runtime-commit-live-policy-report" not in doc_text:
        print(
            "expected Kolme devnet ops doc to document process lifecycle runtime policy report pass-through option",
            file=sys.stderr,
        )
        return 1
    # Regression: #2107
    if "--rollback-evidence-file" not in doc_text:
        print(
            "expected Kolme devnet ops doc to document process lifecycle rollback evidence option",
            file=sys.stderr,
        )
        return 1
    if "--recovery-evidence-file" not in doc_text:
        print(
            "expected Kolme devnet ops doc to document process lifecycle recovery evidence option",
            file=sys.stderr,
        )
        return 1
    if "run_local_kamn_live_runtime_integration_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local KAMN live runtime integration prerequisite lane", file=sys.stderr)
        return 1
    if "Regression: #1494" not in doc_text:
        print("expected Kolme devnet ops doc to include local fork process lifecycle regression marker", file=sys.stderr)
        return 1
    if "Regression: #1973" not in doc_text:
        print(
            "expected Kolme devnet ops doc to include process lifecycle integration finality pass-through regression marker",
            file=sys.stderr,
        )
        return 1
    if "run_local_kolme_fork_process_lifecycle_contract_lane.sh" not in readme_text:
        print("expected README to reference local Kolme fork process lifecycle contract lane", file=sys.stderr)
        return 1
    if "--integration-runtime-commit-finality-command" not in readme_text:
        print(
            "expected README to document process lifecycle integration finality pass-through option",
            file=sys.stderr,
        )
        return 1
    if "--integration-runtime-commit-live-policy-report" not in readme_text:
        print(
            "expected README to document process lifecycle runtime policy report pass-through option",
            file=sys.stderr,
        )
        return 1
    if "--rollback-evidence-file" not in readme_text:
        print(
            "expected README to document process lifecycle rollback evidence option",
            file=sys.stderr,
        )
        return 1
    if "--recovery-evidence-file" not in readme_text:
        print(
            "expected README to document process lifecycle recovery evidence option",
            file=sys.stderr,
        )
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"local Kolme fork process lifecycle contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local Kolme fork process lifecycle contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
