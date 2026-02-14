#!/usr/bin/env python3
"""Contract lane for deterministic on-chain lifecycle evidence composition."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time

# Regression: #3249

ROOT_DIR = Path(__file__).resolve().parents[3]
BUNDLE_RUNNER = ROOT_DIR / "scripts/kolme/run_onchain_lifecycle_evidence_bundle_lane.sh"
POLICY_TOOL = ROOT_DIR / "scripts/kolme/check_onchain_lifecycle_evidence_policy.py"
CONTRACT_RUNNER = ROOT_DIR / "scripts/kolme/run_onchain_lifecycle_evidence_contract_lane.sh"
DOC_FILE = ROOT_DIR / "docs/foundation/kolme-runtime-commit-client.md"
OPS_DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE = ROOT_DIR / "docs/ci/strategy.md"
README_FILE = ROOT_DIR / "README.md"

# Live integration markers ensure command-surface parity coverage for the three
# underlying on-chain validators:
# - scripts/kolme/validate_did_lifecycle_chain_adapter_live.sh
# - scripts/kolme/validate_message_proof_anchoring_live.sh
# - scripts/kolme/validate_continuous_runtime_commit_live.sh
LIVE_VALIDATION_MARKERS = (
    "validate_did_lifecycle_chain_adapter_live.sh",
    "validate_message_proof_anchoring_live.sh",
    "validate_continuous_runtime_commit_live.sh",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run on-chain lifecycle evidence contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-onchain-lifecycle-evidence-summary.json",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-onchain-lifecycle-evidence-policy.json",
    )
    parser.add_argument(
        "--max-seconds",
        default="180",
    )
    parser.add_argument(
        "--run-live-integration",
        action="store_true",
        help="Run optional local-heavy integration drill against real live validation scripts.",
    )
    return parser.parse_args()


def load_json(path: Path) -> dict[str, object]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def ensure_reason_code(policy_path: Path, marker: str) -> None:
    policy_payload = load_json(policy_path)
    reason_codes = policy_payload.get("reason_codes")
    if not isinstance(reason_codes, list):
        raise SystemExit("expected reason_codes list in policy payload")
    if marker not in reason_codes:
        raise SystemExit(f"expected policy reason marker: {marker}")


def run_command(command: list[str], *, expected_success: bool) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        command,
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
        check=False,
    )
    if expected_success and proc.returncode != 0:
        if proc.stdout:
            print(proc.stdout, file=sys.stderr, end="")
        if proc.stderr:
            print(proc.stderr, file=sys.stderr, end="")
        raise SystemExit(f"command failed unexpectedly: {' '.join(command)}")
    if not expected_success and proc.returncode == 0:
        raise SystemExit(f"expected command to fail closed: {' '.join(command)}")
    return proc


def ensure_docs_markers() -> None:
    doc_files = (DOC_FILE, OPS_DOC_FILE, CI_DOC_FILE, README_FILE)
    required_markers = (
        "run_onchain_lifecycle_evidence_bundle_lane.sh",
        "check_onchain_lifecycle_evidence_policy.py",
        "run_onchain_lifecycle_evidence_contract_lane.sh",
        "aggregate_bundle_lineage_mismatch",
        "finality_lineage_missing",
        "recovery_lineage_missing",
        "Regression: #3249",
    )
    for doc_file in doc_files:
        text = doc_file.read_text(encoding="utf-8")
        for marker in required_markers:
            if marker not in text:
                raise SystemExit(f"expected docs parity marker '{marker}' in {doc_file}")


def main() -> int:
    args = parse_args()
    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    max_seconds = int(args.max_seconds)

    if not BUNDLE_RUNNER.is_file() or not BUNDLE_RUNNER.stat().st_mode & 0o111:
        print("expected on-chain lifecycle bundle runner to be executable", file=sys.stderr)
        return 1
    if not CONTRACT_RUNNER.is_file() or not CONTRACT_RUNNER.stat().st_mode & 0o111:
        print("expected on-chain lifecycle contract runner to be executable", file=sys.stderr)
        return 1
    if not POLICY_TOOL.is_file() or not POLICY_TOOL.stat().st_mode & 0o111:
        print("expected on-chain lifecycle policy tool to be executable", file=sys.stderr)
        return 1
    for doc_file in (DOC_FILE, OPS_DOC_FILE, CI_DOC_FILE, README_FILE):
        if not doc_file.is_file():
            print(f"expected docs file to exist: {doc_file}", file=sys.stderr)
            return 1

    start_epoch = time.monotonic()
    with tempfile.TemporaryDirectory(prefix="onchain-lifecycle-contract-") as temp_dir:
        temp_path = Path(temp_dir)
        did_report = temp_path / "did.json"
        message_report = temp_path / "message.json"
        runtime_report = temp_path / "runtime.json"
        summary_path = Path(args.output_json).resolve()
        policy_path = Path(args.policy_output_json).resolve()

        run_command(
            [
                "bash",
                str(BUNDLE_RUNNER),
                "--mode",
                "dry-run",
                "--did-report-file",
                str(did_report),
                "--message-report-file",
                str(message_report),
                "--runtime-report-file",
                str(runtime_report),
                "--max-seconds",
                str(max_seconds),
                "--output-json",
                str(summary_path),
            ],
            expected_success=True,
        )

        run_command(
            [
                "python3",
                str(POLICY_TOOL),
                "check",
                "--report-file",
                str(summary_path),
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                "PASS",
                "--require-reason-code",
                "dry_run_no_commands_executed",
                "--output-json",
                str(policy_path),
            ],
            expected_success=True,
        )

        summary = load_json(summary_path)
        policy = load_json(policy_path)
        if summary.get("schema_version") != "kamn.kolme.onchain-lifecycle-evidence-bundle.v1":
            raise SystemExit("unexpected on-chain lifecycle bundle schema")
        if summary.get("status") != "ok":
            raise SystemExit("expected on-chain lifecycle bundle status ok")
        if summary.get("final_decision") != "GO":
            raise SystemExit("expected on-chain lifecycle bundle final_decision GO")
        if summary.get("finality_lineage_status") != "verified":
            raise SystemExit("expected on-chain lifecycle finality lineage marker")
        if summary.get("recovery_lineage_status") != "verified":
            raise SystemExit("expected on-chain lifecycle recovery lineage marker")
        linked_artifacts = summary.get("linked_artifacts")
        if not isinstance(linked_artifacts, list) or len(linked_artifacts) != 3:
            raise SystemExit("expected three linked artifacts in on-chain lifecycle bundle")
        if policy.get("schema_version") != "kamn.kolme.onchain-lifecycle-evidence-policy-report.v1":
            raise SystemExit("unexpected on-chain lifecycle policy schema")
        if policy.get("final_decision") != "GO":
            raise SystemExit("expected on-chain lifecycle policy final_decision GO")

        missing_finality_runtime = load_json(runtime_report)
        missing_finality_runtime["continuous_runtime_contract_status"] = "missing"
        runtime_missing_finality_path = temp_path / "runtime-missing-finality.json"
        write_json(runtime_missing_finality_path, missing_finality_runtime)
        missing_finality_summary_path = temp_path / "missing-finality-summary.json"
        missing_finality_policy_path = temp_path / "missing-finality-policy.json"
        run_command(
            [
                "python3",
                str(POLICY_TOOL),
                "generate",
                "--mode",
                "run",
                "--did-report-file",
                str(did_report),
                "--message-report-file",
                str(message_report),
                "--runtime-report-file",
                str(runtime_missing_finality_path),
                "--max-seconds",
                str(max_seconds),
                "--elapsed-seconds",
                "1",
                "--budget-status",
                "within_budget",
                "--reason-code",
                "live_onchain_lifecycle_bundle_passed",
                "--output-json",
                str(missing_finality_summary_path),
            ],
            expected_success=False,
        )
        run_command(
            [
                "python3",
                str(POLICY_TOOL),
                "check",
                "--report-file",
                str(missing_finality_summary_path),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(missing_finality_policy_path),
            ],
            expected_success=False,
        )
        ensure_reason_code(missing_finality_policy_path, "finality_lineage_missing:runtime_commit")

        missing_recovery_did = load_json(did_report)
        missing_recovery_did["fail_closed_status"] = "missing"
        did_missing_recovery_path = temp_path / "did-missing-recovery.json"
        write_json(did_missing_recovery_path, missing_recovery_did)
        missing_recovery_summary_path = temp_path / "missing-recovery-summary.json"
        missing_recovery_policy_path = temp_path / "missing-recovery-policy.json"
        run_command(
            [
                "python3",
                str(POLICY_TOOL),
                "generate",
                "--mode",
                "run",
                "--did-report-file",
                str(did_missing_recovery_path),
                "--message-report-file",
                str(message_report),
                "--runtime-report-file",
                str(runtime_report),
                "--max-seconds",
                str(max_seconds),
                "--elapsed-seconds",
                "1",
                "--budget-status",
                "within_budget",
                "--reason-code",
                "live_onchain_lifecycle_bundle_passed",
                "--output-json",
                str(missing_recovery_summary_path),
            ],
            expected_success=False,
        )
        run_command(
            [
                "python3",
                str(POLICY_TOOL),
                "check",
                "--report-file",
                str(missing_recovery_summary_path),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(missing_recovery_policy_path),
            ],
            expected_success=False,
        )
        ensure_reason_code(missing_recovery_policy_path, "recovery_lineage_missing:did_lifecycle")

        tampered_summary = load_json(summary_path)
        linked = tampered_summary.get("linked_artifacts", [])
        if not isinstance(linked, list) or not linked:
            raise SystemExit("expected linked artifacts in baseline bundle for tamper drill")
        first_artifact = linked[0]
        if not isinstance(first_artifact, dict):
            raise SystemExit("expected linked artifact object in tamper drill")
        first_artifact["sha256"] = "0" * 64
        tampered_summary_path = temp_path / "tampered-summary.json"
        tampered_policy_path = temp_path / "tampered-policy.json"
        write_json(tampered_summary_path, tampered_summary)
        run_command(
            [
                "python3",
                str(POLICY_TOOL),
                "check",
                "--report-file",
                str(tampered_summary_path),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(tampered_policy_path),
            ],
            expected_success=False,
        )
        ensure_reason_code(tampered_policy_path, "aggregate_bundle_lineage_mismatch")

        if args.run_live_integration:
            run_command(
                [
                    "bash",
                    str(BUNDLE_RUNNER),
                    "--mode",
                    "run",
                    "--did-report-file",
                    str(temp_path / "live-did.json"),
                    "--message-report-file",
                    str(temp_path / "live-message.json"),
                    "--runtime-report-file",
                    str(temp_path / "live-runtime.json"),
                    "--max-seconds",
                    str(max_seconds),
                    "--output-json",
                    str(temp_path / "live-summary.json"),
                ],
                expected_success=True,
            )
            run_command(
                [
                    "python3",
                    str(POLICY_TOOL),
                    "check",
                    "--report-file",
                    str(temp_path / "live-summary.json"),
                    "--expected-final-decision",
                    "GO",
                    "--ci-fast-gate",
                    "PASS",
                    "--output-json",
                    str(temp_path / "live-policy.json"),
                ],
                expected_success=True,
            )

    ensure_docs_markers()

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"on-chain lifecycle evidence contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("status=pass")
    print("final_decision=GO")
    print("bundle_contract_status=verified")
    print("policy_contract_status=verified")
    print("tamper_fail_closed_status=verified")
    print("finality_lineage_fail_closed_status=verified")
    print("recovery_lineage_fail_closed_status=verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
