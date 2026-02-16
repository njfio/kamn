#!/usr/bin/env python3
"""Contract lane runner for local runtime-commit submit/finality evidence checks."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_runtime_commit_live_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_runtime_commit_live_evidence_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
FOUNDATION_DOC = ROOT_DIR / "docs/foundation/kolme-runtime-commit-client.md"
CI_STRATEGY_DOC = ROOT_DIR / "docs/ci/strategy.md"
README_FILE = ROOT_DIR / "README.md"
SUBMIT_FINALITY_REASON_TAXONOMY_VERSION = (
    "kamn.kolme.local-runtime-commit-submit-finality-reason-taxonomy.v1"
)
SUBMIT_FINALITY_REASON_CODES_CSV = (
    "submit_finality_reason_mismatch_for_finality_enabled_run,"
    "submit_finality_reason_mismatch_for_submit_only_run"
)


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
    parser.add_argument(
        "--finality-retry-max-attempts",
        default="2",
        help="Bounded retry attempts for finality command execution.",
    )
    parser.add_argument(
        "--finality-retry-backoff-seconds",
        default="0",
        help="Retry backoff in seconds between finality command attempts.",
    )
    parser.add_argument(
        "--expected-provider-client-contract",
        default="KolmeRuntimeCommitLiveProvider",
        help="Expected provider client contract emitted by runtime live lane summary.",
    )
    parser.add_argument(
        "--require-non-synthetic-run-evidence",
        action="store_true",
        help="Require fail-closed non-synthetic run evidence checks in policy evaluation.",
    )
    parser.add_argument(
        "--require-native-payload-evidence",
        action="store_true",
        help="Require fail-closed native payload marker checks in policy evaluation.",
    )
    return parser


def _is_positive_integer(raw_value: str) -> bool:
    return raw_value.isdigit() and int(raw_value) > 0


def _is_non_negative_integer(raw_value: str) -> bool:
    return raw_value.isdigit() and int(raw_value) >= 0


def main() -> int:
    args = build_parser().parse_args()

    if not _is_positive_integer(args.max_seconds):
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    if not _is_positive_integer(args.finality_max_seconds):
        print("finality-max-seconds must be a positive integer", file=sys.stderr)
        return 1
    if not _is_positive_integer(args.finality_retry_max_attempts):
        print("finality-retry-max-attempts must be a positive integer", file=sys.stderr)
        return 1
    if not _is_non_negative_integer(args.finality_retry_backoff_seconds):
        print("finality-retry-backoff-seconds must be a non-negative integer", file=sys.stderr)
        return 1
    if not args.expected_provider_client_contract.strip():
        print("expected-provider-client-contract must not be empty", file=sys.stderr)
        return 1

    max_seconds = int(args.max_seconds)
    finality_max_seconds = int(args.finality_max_seconds)
    finality_retry_max_attempts = int(args.finality_retry_max_attempts)
    finality_retry_backoff_seconds = int(args.finality_retry_backoff_seconds)

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
        "replay_evidence_marker_present",
        "replay_evidence_contract_version",
        "request_payload_evidence_marker_present",
        "request_payload_evidence_artifact_path",
        "submit_evidence_artifact_path",
        "finality_evidence_artifact_path",
        "request_finality_evidence_contract_version",
        "request_finality_evidence_linked",
        "replay_evidence_marker_missing",
        "request_payload_evidence_marker_missing",
        "finality_evidence_artifact_path_missing",
        "request_finality_evidence_linkage_missing",
        "finality_retry_contract_version",
        "finality_retry_max_attempts",
        "finality_retry_backoff_seconds",
        "finality_retry_attempts_used",
        "finality_retry_exhausted",
        "finality_retry_failure_class",
        "live_finality_retry_exhausted_timeout",
        "live_finality_retry_exhausted_failed",
        "finality_retry_failure_class_mismatch_for_timeout_reason",
        "finality_retry_attempts_used_mismatch_for_timeout_reason",
        "submit_finality_reason_taxonomy_version",
        "submit_finality_reason_codes_csv",
        "submit_finality_reason_codes_value",
        "submit_finality_reason_mismatch_for_finality_enabled_run",
        "submit_finality_reason_mismatch_for_submit_only_run",
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
            "--finality-retry-max-attempts",
            str(finality_retry_max_attempts),
            "--finality-retry-backoff-seconds",
            str(finality_retry_backoff_seconds),
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
            "--expected-provider-client-contract",
            args.expected_provider_client_contract,
            "--output-json",
            args.policy_output_json,
            *(
                ["--require-non-synthetic-run-evidence"]
                if args.require_non_synthetic_run_evidence
                else []
            ),
            *(
                ["--require-native-payload-evidence"]
                if args.require_native_payload_evidence
                else []
            ),
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
            "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 printf 'status=submitted\\nintegration_kolme_fork_live_node_submit_reaches_endpoint\\nreplay_guard=verified\\n{\"pubkey\":\"proof\",\"nonce\":1,\"messages\":[]}\\n'",
            "--finality-command",
            "printf 'finality=final\\n'",
            "--finality-retry-max-attempts",
            str(finality_retry_max_attempts),
            "--finality-retry-backoff-seconds",
            str(finality_retry_backoff_seconds),
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
            "--expected-provider-client-contract",
            args.expected_provider_client_contract,
            "--output-json",
            args.policy_output_json,
            *(
                ["--require-non-synthetic-run-evidence"]
                if args.require_non_synthetic_run_evidence
                else []
            ),
            *(
                ["--require-native-payload-evidence"]
                if args.require_native_payload_evidence
                else []
            ),
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    summary_payload = json_load(Path(args.output_json))
    if summary_payload.get("schema_version") != "kamn.kolme.local-runtime-commit-live-summary.v1":
        print("unexpected runtime-commit live summary schema", file=sys.stderr)
        return 1
    if summary_payload.get("provider_contract_enforcement_mode") != "live-provider-only-v1":
        print("expected provider_contract_enforcement_mode=live-provider-only-v1", file=sys.stderr)
        return 1
    expected_provider_live_contract_marker = (
        f"provider_client_contract={args.expected_provider_client_contract}"
    )
    if summary_payload.get("provider_live_contract_marker") != expected_provider_live_contract_marker:
        print("expected deterministic provider_live_contract_marker in summary", file=sys.stderr)
        return 1
    if summary_payload.get("provider_live_contract_marker_present") is not True:
        print("expected provider_live_contract_marker_present=true in summary", file=sys.stderr)
        return 1
    if summary_payload.get("provider_in_memory_reference_detected") is not False:
        print("expected provider_in_memory_reference_detected=false in summary", file=sys.stderr)
        return 1
    if summary_payload.get("provider_signer_adapter_contract") != "KolmeForkSecp256k1SignerAdapter":
        print(
            "expected provider_signer_adapter_contract=KolmeForkSecp256k1SignerAdapter in summary",
            file=sys.stderr,
        )
        return 1
    if summary_payload.get("provider_signing_curve_contract") != "secp256k1":
        print("expected provider_signing_curve_contract=secp256k1 in summary", file=sys.stderr)
        return 1
    if summary_payload.get("provider_signing_profile_contract_version") != "v1":
        print("expected provider_signing_profile_contract_version=v1 in summary", file=sys.stderr)
        return 1
    if summary_payload.get("submit_evidence_marker_present") is not True:
        print("expected submit_evidence_marker_present=true", file=sys.stderr)
        return 1
    if summary_payload.get("finality_evidence_marker_present") is not True:
        print("expected finality_evidence_marker_present=true", file=sys.stderr)
        return 1
    if summary_payload.get("replay_evidence_marker") != "replay_guard=verified":
        print("expected replay_evidence_marker", file=sys.stderr)
        return 1
    if summary_payload.get("replay_evidence_marker_present") is not True:
        print("expected replay_evidence_marker_present=true", file=sys.stderr)
        return 1
    if summary_payload.get("replay_evidence_contract_version") != "v1":
        print("expected replay_evidence_contract_version=v1", file=sys.stderr)
        return 1
    if summary_payload.get("request_payload_evidence_marker") != "native_payload_pubkey_nonce_messages":
        print("expected request_payload_evidence_marker", file=sys.stderr)
        return 1
    if summary_payload.get("request_payload_evidence_marker_present") is not True:
        print("expected request_payload_evidence_marker_present=true", file=sys.stderr)
        return 1
    if summary_payload.get("request_payload_evidence_artifact_path") != summary_payload.get("live_output_file"):
        print("expected request_payload_evidence_artifact_path to match live_output_file", file=sys.stderr)
        return 1
    if summary_payload.get("submit_evidence_artifact_path") != summary_payload.get("live_output_file"):
        print("expected submit_evidence_artifact_path to match live_output_file", file=sys.stderr)
        return 1
    if summary_payload.get("finality_evidence_artifact_path") != summary_payload.get("finality_output_file"):
        print("expected finality_evidence_artifact_path to match finality_output_file", file=sys.stderr)
        return 1
    if summary_payload.get("request_finality_evidence_contract_version") != "v1":
        print("expected request_finality_evidence_contract_version=v1", file=sys.stderr)
        return 1
    if summary_payload.get("request_finality_evidence_linked") is not True:
        print("expected request_finality_evidence_linked=true", file=sys.stderr)
        return 1
    if summary_payload.get("finality_retry_contract_version") != "v1":
        print("expected finality_retry_contract_version=v1", file=sys.stderr)
        return 1
    if summary_payload.get("finality_retry_max_attempts") != finality_retry_max_attempts:
        print("expected finality_retry_max_attempts marker in summary", file=sys.stderr)
        return 1
    if summary_payload.get("finality_retry_backoff_seconds") != finality_retry_backoff_seconds:
        print("expected finality_retry_backoff_seconds marker in summary", file=sys.stderr)
        return 1
    if summary_payload.get("finality_retry_attempts_used") != 1:
        print("expected finality_retry_attempts_used=1 marker in summary", file=sys.stderr)
        return 1
    if summary_payload.get("finality_retry_exhausted") is not False:
        print("expected finality_retry_exhausted=false marker in summary", file=sys.stderr)
        return 1
    if summary_payload.get("finality_retry_failure_class") != "none":
        print("expected finality_retry_failure_class=none marker in summary", file=sys.stderr)
        return 1
    if summary_payload.get("native_payload_pubkey_marker_present") is not True:
        print("expected native_payload_pubkey_marker_present=true", file=sys.stderr)
        return 1
    if summary_payload.get("native_payload_nonce_marker_present") is not True:
        print("expected native_payload_nonce_marker_present=true", file=sys.stderr)
        return 1
    if summary_payload.get("native_payload_messages_marker_present") is not True:
        print("expected native_payload_messages_marker_present=true", file=sys.stderr)
        return 1

    policy_payload = json_load(Path(args.policy_output_json))
    if policy_payload.get("schema_version") != "kamn.kolme.local-runtime-commit-live-policy-report.v1":
        print("unexpected runtime-commit live finality evidence policy schema", file=sys.stderr)
        return 1
    if policy_payload.get("final_decision") != "GO":
        print(
            "expected runtime-commit live finality evidence policy final_decision GO",
            file=sys.stderr,
        )
        return 1
    if (
        policy_payload.get("submit_finality_reason_taxonomy_version")
        != SUBMIT_FINALITY_REASON_TAXONOMY_VERSION
    ):
        print(
            "expected submit_finality_reason_taxonomy_version in policy output",
            file=sys.stderr,
        )
        return 1
    if policy_payload.get("submit_finality_reason_codes_csv") != SUBMIT_FINALITY_REASON_CODES_CSV:
        print(
            "expected submit_finality_reason_codes_csv in policy output",
            file=sys.stderr,
        )
        return 1
    if policy_payload.get("submit_finality_reason_codes_value") != "none":
        print(
            "expected submit_finality_reason_codes_value=none in policy output",
            file=sys.stderr,
        )
        return 1

    with tempfile.TemporaryDirectory(prefix="runtime-commit-finality-negative-") as temp_dir:
        negative_root = Path(temp_dir)
        linkage_drift_summary_file = negative_root / "linkage_drift_summary.json"
        linkage_drift_policy_file = negative_root / "linkage_drift_policy.json"

        linkage_drift_summary = dict(summary_payload)
        linkage_drift_summary["request_finality_evidence_linked"] = False
        linkage_drift_summary["finality_evidence_artifact_path"] = "/tmp/missing-runtime-finality-artifact.txt"
        linkage_drift_summary["replay_evidence_marker_present"] = False
        linkage_drift_summary["request_payload_evidence_marker_present"] = False
        linkage_drift_summary_file.write_text(
            json.dumps(linkage_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        linkage_drift_result = subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                str(linkage_drift_summary_file),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--expected-provider-client-contract",
                args.expected_provider_client_contract,
                "--require-native-payload-evidence",
                "--output-json",
                str(linkage_drift_policy_file),
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if linkage_drift_result.returncode == 0:
            print("expected request/finality linkage drift proof to fail closed", file=sys.stderr)
            return 1
        linkage_drift_policy = json.loads(linkage_drift_policy_file.read_text(encoding="utf-8"))
        linkage_drift_reason_codes = linkage_drift_policy.get("reason_codes")
        if not isinstance(linkage_drift_reason_codes, list):
            print("expected reason_codes list in linkage drift policy output", file=sys.stderr)
            return 1
        if "request_finality_evidence_linkage_missing" not in linkage_drift_reason_codes:
            print(
                "expected request_finality_evidence_linkage_missing in linkage drift policy output",
                file=sys.stderr,
            )
            return 1
        if "finality_evidence_artifact_path_missing" not in linkage_drift_reason_codes:
            print(
                "expected finality_evidence_artifact_path_missing in linkage drift policy output",
                file=sys.stderr,
            )
            return 1
        if "request_payload_evidence_marker_missing" not in linkage_drift_reason_codes:
            print(
                "expected request_payload_evidence_marker_missing in linkage drift policy output",
                file=sys.stderr,
            )
            return 1
        if "replay_evidence_marker_missing" not in linkage_drift_reason_codes:
            print(
                "expected replay_evidence_marker_missing in linkage drift policy output",
                file=sys.stderr,
            )
            return 1

        provider_drift_summary_file = negative_root / "provider_drift_summary.json"
        provider_drift_policy_file = negative_root / "provider_drift_policy.json"
        provider_drift_summary = dict(summary_payload)
        provider_drift_summary["provider_in_memory_reference_detected"] = True
        provider_drift_summary_file.write_text(
            json.dumps(provider_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        provider_drift_result = subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                str(provider_drift_summary_file),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--expected-provider-client-contract",
                args.expected_provider_client_contract,
                "--output-json",
                str(provider_drift_policy_file),
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if provider_drift_result.returncode == 0:
            print("expected in-memory provider drift proof to fail closed", file=sys.stderr)
            return 1
        provider_drift_policy = json.loads(provider_drift_policy_file.read_text(encoding="utf-8"))
        provider_drift_reason_codes = provider_drift_policy.get("reason_codes")
        if not isinstance(provider_drift_reason_codes, list):
            print("expected reason_codes list in provider drift policy output", file=sys.stderr)
            return 1
        if "provider_in_memory_reference_detected" not in provider_drift_reason_codes:
            print(
                "expected provider_in_memory_reference_detected in provider drift policy output",
                file=sys.stderr,
            )
            return 1

        signer_adapter_drift_summary_file = negative_root / "signer_adapter_drift_summary.json"
        signer_adapter_drift_policy_file = negative_root / "signer_adapter_drift_policy.json"
        signer_adapter_drift_summary = dict(summary_payload)
        signer_adapter_drift_summary["provider_signer_adapter_contract"] = "SimulatedSignerAdapter"
        signer_adapter_drift_summary_file.write_text(
            json.dumps(signer_adapter_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        signer_adapter_drift_result = subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                str(signer_adapter_drift_summary_file),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--expected-provider-client-contract",
                args.expected_provider_client_contract,
                "--output-json",
                str(signer_adapter_drift_policy_file),
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if signer_adapter_drift_result.returncode == 0:
            print("expected signer adapter drift proof to fail closed", file=sys.stderr)
            return 1
        signer_adapter_drift_policy = json.loads(
            signer_adapter_drift_policy_file.read_text(encoding="utf-8")
        )
        signer_adapter_drift_reason_codes = signer_adapter_drift_policy.get("reason_codes")
        if not isinstance(signer_adapter_drift_reason_codes, list):
            print("expected reason_codes list in signer adapter drift policy output", file=sys.stderr)
            return 1
        if "provider_signer_adapter_contract_mismatch" not in signer_adapter_drift_reason_codes:
            print(
                "expected provider_signer_adapter_contract_mismatch in signer adapter drift policy output",
                file=sys.stderr,
            )
            return 1

        retry_drift_summary_file = negative_root / "retry_drift_summary.json"
        retry_drift_policy_file = negative_root / "retry_drift_policy.json"
        retry_drift_summary = dict(summary_payload)
        retry_drift_summary["status"] = "fail"
        retry_drift_summary["reason_code"] = "live_finality_retry_exhausted_timeout"
        retry_drift_summary["finality_evidence_marker_present"] = False
        retry_drift_summary["finality_retry_attempts_used"] = finality_retry_max_attempts
        retry_drift_summary["finality_retry_exhausted"] = True
        retry_drift_summary["finality_retry_failure_class"] = "failed"
        retry_drift_summary_file.write_text(
            json.dumps(retry_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        retry_drift_result = subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                str(retry_drift_summary_file),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--expected-provider-client-contract",
                args.expected_provider_client_contract,
                "--require-reason-code",
                "live_finality_retry_exhausted_timeout",
                "--output-json",
                str(retry_drift_policy_file),
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if retry_drift_result.returncode == 0:
            print("expected retry drift proof to fail closed", file=sys.stderr)
            return 1
        retry_drift_policy = json.loads(retry_drift_policy_file.read_text(encoding="utf-8"))
        retry_drift_reason_codes = retry_drift_policy.get("reason_codes")
        if not isinstance(retry_drift_reason_codes, list):
            print("expected reason_codes list in retry drift policy output", file=sys.stderr)
            return 1
        if "finality_retry_failure_class_mismatch_for_timeout_reason" not in retry_drift_reason_codes:
            print(
                "expected finality_retry_failure_class_mismatch_for_timeout_reason in retry drift policy output",
                file=sys.stderr,
            )
            return 1
        if (
            retry_drift_policy.get("submit_finality_reason_taxonomy_version")
            != SUBMIT_FINALITY_REASON_TAXONOMY_VERSION
        ):
            print(
                "expected submit/finality taxonomy version in retry drift policy output",
                file=sys.stderr,
            )
            return 1
        if retry_drift_policy.get("submit_finality_reason_codes_csv") != SUBMIT_FINALITY_REASON_CODES_CSV:
            print(
                "expected submit/finality taxonomy ordering in retry drift policy output",
                file=sys.stderr,
            )
            return 1
        if retry_drift_policy.get("submit_finality_reason_codes_value") != "none":
            print(
                "expected submit/finality taxonomy value=none in retry drift policy output",
                file=sys.stderr,
            )
            return 1

        submit_finality_drift_summary_file = negative_root / "submit_finality_drift_summary.json"
        submit_finality_drift_policy_file = negative_root / "submit_finality_drift_policy.json"
        submit_finality_drift_summary = dict(summary_payload)
        submit_finality_drift_summary["status"] = "ok"
        submit_finality_drift_summary["reason_code"] = "live_runtime_commit_command_passed"
        submit_finality_drift_summary["finality_enabled"] = True
        submit_finality_drift_summary["finality_evidence_marker_present"] = True
        submit_finality_drift_summary["finality_retry_attempts_used"] = 1
        submit_finality_drift_summary["finality_retry_exhausted"] = False
        submit_finality_drift_summary["finality_retry_failure_class"] = "none"
        submit_finality_drift_summary_file.write_text(
            json.dumps(submit_finality_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        submit_finality_drift_result = subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                str(submit_finality_drift_summary_file),
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                "PASS",
                "--expected-provider-client-contract",
                args.expected_provider_client_contract,
                "--output-json",
                str(submit_finality_drift_policy_file),
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if submit_finality_drift_result.returncode == 0:
            print("expected submit/finality mismatch proof to fail closed", file=sys.stderr)
            return 1
        submit_finality_drift_policy = json.loads(
            submit_finality_drift_policy_file.read_text(encoding="utf-8")
        )
        submit_finality_drift_reason_codes = submit_finality_drift_policy.get("reason_codes")
        if not isinstance(submit_finality_drift_reason_codes, list):
            print(
                "expected reason_codes list in submit/finality mismatch policy output",
                file=sys.stderr,
            )
            return 1
        if (
            "submit_finality_reason_mismatch_for_finality_enabled_run"
            not in submit_finality_drift_reason_codes
        ):
            print(
                "expected submit_finality_reason_mismatch_for_finality_enabled_run in policy output",
                file=sys.stderr,
            )
            return 1
        if (
            submit_finality_drift_policy.get("submit_finality_reason_codes_value")
            != "submit_finality_reason_mismatch_for_finality_enabled_run"
        ):
            print(
                "expected normalized submit/finality taxonomy value in mismatch output",
                file=sys.stderr,
            )
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
    return json.loads(path.read_text(encoding="utf-8"))


if __name__ == "__main__":
    raise SystemExit(main())
