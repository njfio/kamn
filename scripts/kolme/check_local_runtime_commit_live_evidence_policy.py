#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate local runtime-commit live lane evidence policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument(
        "--expected-provider-client-contract",
        default="KolmeRuntimeCommitLiveProvider",
    )
    parser.add_argument(
        "--require-non-synthetic-run-evidence",
        action="store_true",
        help="Fail closed when run-mode evidence commands are classified as synthetic.",
    )
    parser.add_argument(
        "--require-native-payload-evidence",
        action="store_true",
        help="Fail closed when native payload markers are absent from run-mode evidence.",
    )
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(report: dict[str, object], args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []
    in_memory_provider_marker = "InMemoryKolmeRuntimeCommitClient"

    if report.get("schema_version") != "kamn.kolme.local-runtime-commit-live-summary.v1":
        reason_codes.append("schema_version_mismatch")

    mode = report.get("mode")
    if mode not in ("dry-run", "run"):
        reason_codes.append("mode_invalid")

    status = report.get("status")
    if status not in ("ok", "fail"):
        reason_codes.append("status_invalid")

    reason_code = report.get("reason_code")
    if not isinstance(reason_code, str) or not reason_code.strip():
        reason_codes.append("reason_code_missing")

    if report.get("local_only_enforced") is not True:
        reason_codes.append("local_only_enforced_missing")

    if report.get("provider_client_contract") != args.expected_provider_client_contract:
        reason_codes.append("provider_client_contract_mismatch")

    provider_contract_enforcement_mode = report.get("provider_contract_enforcement_mode")
    if provider_contract_enforcement_mode != "live-provider-only-v1":
        reason_codes.append("provider_contract_enforcement_mode_mismatch")

    expected_provider_live_contract_marker = (
        f"provider_client_contract={args.expected_provider_client_contract}"
    )
    provider_live_contract_marker = report.get("provider_live_contract_marker")
    if provider_live_contract_marker != expected_provider_live_contract_marker:
        reason_codes.append("provider_live_contract_marker_mismatch")

    provider_live_contract_marker_present = report.get("provider_live_contract_marker_present")
    if not isinstance(provider_live_contract_marker_present, bool):
        reason_codes.append("provider_live_contract_marker_present_invalid")
    elif provider_live_contract_marker_present is False:
        reason_codes.append("provider_live_contract_marker_missing")

    provider_in_memory_reference_detected = report.get("provider_in_memory_reference_detected")
    if not isinstance(provider_in_memory_reference_detected, bool):
        reason_codes.append("provider_in_memory_reference_detected_invalid")
    elif provider_in_memory_reference_detected is True:
        reason_codes.append("provider_in_memory_reference_detected")

    provider_hint = report.get("provider_hint")
    if not isinstance(provider_hint, str) or not provider_hint.strip():
        reason_codes.append("provider_hint_missing")
    elif in_memory_provider_marker in provider_hint:
        reason_codes.append("provider_hint_in_memory_provider_reference_detected")

    if report.get("provider_submit_profile_contract") != "kolme_fork_broadcast_profile":
        reason_codes.append("provider_submit_profile_contract_mismatch")

    if (
        report.get("provider_command_marker")
        != "integration_kolme_fork_live_node_submit_reaches_endpoint"
    ):
        reason_codes.append("provider_command_marker_mismatch")

    provider_command_marker_present = report.get("provider_command_marker_present")
    if not isinstance(provider_command_marker_present, bool):
        reason_codes.append("provider_command_marker_present_invalid")
    elif provider_command_marker_present is False:
        reason_codes.append("provider_command_marker_missing")

    if (
        report.get("provider_signing_profile_marker")
        != "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1"
    ):
        reason_codes.append("provider_signing_profile_marker_mismatch")

    provider_signing_profile_marker_present = report.get("provider_signing_profile_marker_present")
    if not isinstance(provider_signing_profile_marker_present, bool):
        reason_codes.append("provider_signing_profile_marker_present_invalid")
    elif provider_signing_profile_marker_present is False:
        reason_codes.append("provider_signing_profile_marker_missing")

    live_command = report.get("live_command")
    if not isinstance(live_command, str) or not live_command.strip():
        reason_codes.append("live_command_missing")
    elif in_memory_provider_marker in live_command:
        reason_codes.append("live_command_in_memory_provider_reference_detected")
    elif "KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated" in live_command:
        reason_codes.append("provider_signing_profile_simulated_detected")

    if report.get("submit_evidence_marker") != "status=submitted":
        reason_codes.append("submit_evidence_marker_mismatch")

    submit_evidence_marker_present = report.get("submit_evidence_marker_present")
    if not isinstance(submit_evidence_marker_present, bool):
        reason_codes.append("submit_evidence_marker_present_invalid")

    if report.get("finality_evidence_marker") != "finality=final":
        reason_codes.append("finality_evidence_marker_mismatch")

    finality_evidence_marker_present = report.get("finality_evidence_marker_present")
    if not isinstance(finality_evidence_marker_present, bool):
        reason_codes.append("finality_evidence_marker_present_invalid")

    finality_enabled = report.get("finality_enabled")
    if not isinstance(finality_enabled, bool):
        reason_codes.append("finality_enabled_invalid")

    if report.get("finality_retry_contract_version") != "v1":
        reason_codes.append("finality_retry_contract_version_mismatch")

    finality_retry_max_attempts = report.get("finality_retry_max_attempts")
    if not isinstance(finality_retry_max_attempts, int) or finality_retry_max_attempts <= 0:
        reason_codes.append("finality_retry_max_attempts_invalid")

    finality_retry_backoff_seconds = report.get("finality_retry_backoff_seconds")
    if not isinstance(finality_retry_backoff_seconds, int) or finality_retry_backoff_seconds < 0:
        reason_codes.append("finality_retry_backoff_seconds_invalid")

    finality_retry_attempts_used = report.get("finality_retry_attempts_used")
    if not isinstance(finality_retry_attempts_used, int) or finality_retry_attempts_used < 0:
        reason_codes.append("finality_retry_attempts_used_invalid")

    finality_retry_exhausted = report.get("finality_retry_exhausted")
    if not isinstance(finality_retry_exhausted, bool):
        reason_codes.append("finality_retry_exhausted_invalid")

    finality_retry_failure_class = report.get("finality_retry_failure_class")
    if finality_retry_failure_class not in ("none", "timeout", "failed"):
        reason_codes.append("finality_retry_failure_class_invalid")

    if (
        isinstance(finality_retry_attempts_used, int)
        and isinstance(finality_retry_max_attempts, int)
        and finality_retry_attempts_used > finality_retry_max_attempts
    ):
        reason_codes.append("finality_retry_attempts_used_exceeds_max_attempts")

    if report.get("native_payload_pubkey_marker") != '"pubkey"':
        reason_codes.append("native_payload_pubkey_marker_mismatch")
    native_payload_pubkey_marker_present = report.get("native_payload_pubkey_marker_present")
    if not isinstance(native_payload_pubkey_marker_present, bool):
        reason_codes.append("native_payload_pubkey_marker_present_invalid")

    if report.get("native_payload_nonce_marker") != '"nonce"':
        reason_codes.append("native_payload_nonce_marker_mismatch")
    native_payload_nonce_marker_present = report.get("native_payload_nonce_marker_present")
    if not isinstance(native_payload_nonce_marker_present, bool):
        reason_codes.append("native_payload_nonce_marker_present_invalid")

    if report.get("native_payload_messages_marker") != '"messages"':
        reason_codes.append("native_payload_messages_marker_mismatch")
    native_payload_messages_marker_present = report.get("native_payload_messages_marker_present")
    if not isinstance(native_payload_messages_marker_present, bool):
        reason_codes.append("native_payload_messages_marker_present_invalid")

    if report.get("request_payload_evidence_marker") != "native_payload_pubkey_nonce_messages":
        reason_codes.append("request_payload_evidence_marker_mismatch")
    request_payload_evidence_marker_present = report.get("request_payload_evidence_marker_present")
    if not isinstance(request_payload_evidence_marker_present, bool):
        reason_codes.append("request_payload_evidence_marker_present_invalid")

    request_payload_evidence_artifact_path = report.get("request_payload_evidence_artifact_path")
    if (
        not isinstance(request_payload_evidence_artifact_path, str)
        or not request_payload_evidence_artifact_path.strip()
    ):
        reason_codes.append("request_payload_evidence_artifact_path_invalid")

    submit_evidence_artifact_path = report.get("submit_evidence_artifact_path")
    if not isinstance(submit_evidence_artifact_path, str) or not submit_evidence_artifact_path.strip():
        reason_codes.append("submit_evidence_artifact_path_invalid")

    finality_evidence_artifact_path = report.get("finality_evidence_artifact_path")
    if not isinstance(finality_evidence_artifact_path, str):
        reason_codes.append("finality_evidence_artifact_path_invalid")

    if report.get("request_finality_evidence_contract_version") != "v1":
        reason_codes.append("request_finality_evidence_contract_version_mismatch")

    request_finality_evidence_linked = report.get("request_finality_evidence_linked")
    if not isinstance(request_finality_evidence_linked, bool):
        reason_codes.append("request_finality_evidence_linked_invalid")

    if report.get("native_payload_marker_contract_version") != "v1":
        reason_codes.append("native_payload_marker_contract_version_mismatch")

    live_command_synthetic = report.get("live_command_synthetic")
    if not isinstance(live_command_synthetic, bool):
        reason_codes.append("live_command_synthetic_invalid")

    finality_command_synthetic = report.get("finality_command_synthetic")
    if not isinstance(finality_command_synthetic, bool):
        reason_codes.append("finality_command_synthetic_invalid")

    if report.get("synthetic_evidence_classification_version") != "v1":
        reason_codes.append("synthetic_evidence_classification_version_mismatch")

    checks = report.get("checks")
    if not isinstance(checks, list) or not checks:
        reason_codes.append("checks_missing")

    artifact_paths = report.get("artifact_paths")
    if not isinstance(artifact_paths, list) or not all(
        isinstance(path, str) and path.strip() for path in artifact_paths
    ):
        reason_codes.append("artifact_paths_invalid")

    budget_status = report.get("budget_status")
    if budget_status not in ("not_run", "within_budget", "exceeded_budget"):
        reason_codes.append("budget_status_invalid")

    allowed_ok_reason_codes = {
        "dry_run_no_commands_executed",
        "live_runtime_commit_command_passed",
        "live_runtime_commit_and_finality_commands_passed",
    }
    allowed_fail_reason_codes = {
        "local_opt_in_missing",
        "live_preflight_timeout",
        "live_preflight_failed",
        "live_runtime_commit_command_timeout",
        "live_runtime_commit_command_failed",
        "live_finality_retry_exhausted_timeout",
        "live_finality_retry_exhausted_failed",
        # Backward-compatible legacy markers retained for older summaries.
        "live_finality_command_timeout",
        "live_finality_command_failed",
        "live_runtime_commit_budget_exceeded",
    }

    if status == "ok":
        if reason_code not in allowed_ok_reason_codes:
            reason_codes.append("ok_status_reason_code_invalid")
        if mode == "dry-run" and reason_code != "dry_run_no_commands_executed":
            reason_codes.append("dry_run_reason_code_mismatch")
        if mode == "run" and reason_code == "dry_run_no_commands_executed":
            reason_codes.append("run_reason_code_mismatch")
        if mode == "run" and submit_evidence_marker_present is not True:
            reason_codes.append("submit_evidence_marker_missing")
        if mode == "run" and request_payload_evidence_marker_present is not True:
            reason_codes.append("request_payload_evidence_marker_missing")
        if mode == "run" and finality_enabled is True and finality_evidence_marker_present is not True:
            reason_codes.append("finality_evidence_marker_missing")
        if mode == "run" and finality_enabled is True and finality_retry_attempts_used == 0:
            reason_codes.append("finality_retry_attempts_used_missing")
        if mode == "run" and finality_enabled is True and finality_retry_exhausted is True:
            reason_codes.append("finality_retry_exhausted_unexpected_for_ok_status")
        if mode == "run" and finality_enabled is True and finality_retry_failure_class != "none":
            reason_codes.append("finality_retry_failure_class_unexpected_for_ok_status")
        if mode == "run" and finality_enabled is False and finality_retry_attempts_used != 0:
            reason_codes.append("finality_retry_attempts_used_unexpected_without_finality")
        if mode == "run" and finality_enabled is False and finality_retry_exhausted is True:
            reason_codes.append("finality_retry_exhausted_unexpected_without_finality")
        if mode == "run" and finality_enabled is False and finality_retry_failure_class != "none":
            reason_codes.append("finality_retry_failure_class_unexpected_without_finality")
        if (
            mode == "run"
            and isinstance(artifact_paths, list)
            and isinstance(request_payload_evidence_artifact_path, str)
            and request_payload_evidence_artifact_path.strip()
            and request_payload_evidence_artifact_path not in artifact_paths
        ):
            reason_codes.append("request_payload_evidence_artifact_path_missing")
        if (
            mode == "run"
            and isinstance(artifact_paths, list)
            and isinstance(submit_evidence_artifact_path, str)
            and submit_evidence_artifact_path.strip()
            and submit_evidence_artifact_path not in artifact_paths
        ):
            reason_codes.append("submit_evidence_artifact_path_missing")
        if mode == "run" and finality_enabled is True:
            if (
                isinstance(artifact_paths, list)
                and isinstance(finality_evidence_artifact_path, str)
                and finality_evidence_artifact_path.strip()
                and finality_evidence_artifact_path not in artifact_paths
            ):
                reason_codes.append("finality_evidence_artifact_path_missing")
            if request_finality_evidence_linked is not True:
                reason_codes.append("request_finality_evidence_linkage_missing")
        if mode == "run" and args.require_non_synthetic_run_evidence:
            if live_command_synthetic is True:
                reason_codes.append("synthetic_live_command_detected")
            if finality_enabled is True and finality_command_synthetic is True:
                reason_codes.append("synthetic_finality_command_detected")
        if mode == "run" and args.require_native_payload_evidence:
            if native_payload_pubkey_marker_present is not True:
                reason_codes.append("native_payload_pubkey_marker_missing")
            if native_payload_nonce_marker_present is not True:
                reason_codes.append("native_payload_nonce_marker_missing")
            if native_payload_messages_marker_present is not True:
                reason_codes.append("native_payload_messages_marker_missing")
        if budget_status == "exceeded_budget":
            reason_codes.append("ok_status_budget_exceeded")
    elif status == "fail":
        if reason_code in allowed_ok_reason_codes:
            reason_codes.append("fail_status_reason_code_mismatch")
        elif reason_code not in allowed_fail_reason_codes:
            reason_codes.append("fail_status_reason_code_invalid")

        if reason_code == "live_finality_retry_exhausted_timeout":
            if finality_enabled is not True:
                reason_codes.append("finality_retry_timeout_reason_without_finality")
            if finality_retry_exhausted is not True:
                reason_codes.append("finality_retry_exhausted_missing_for_timeout_reason")
            if finality_retry_failure_class != "timeout":
                reason_codes.append("finality_retry_failure_class_mismatch_for_timeout_reason")
            if (
                isinstance(finality_retry_attempts_used, int)
                and isinstance(finality_retry_max_attempts, int)
                and finality_retry_attempts_used != finality_retry_max_attempts
            ):
                reason_codes.append("finality_retry_attempts_used_mismatch_for_timeout_reason")

        if reason_code == "live_finality_retry_exhausted_failed":
            if finality_enabled is not True:
                reason_codes.append("finality_retry_failed_reason_without_finality")
            if finality_retry_exhausted is not True:
                reason_codes.append("finality_retry_exhausted_missing_for_failed_reason")
            if finality_retry_failure_class != "failed":
                reason_codes.append("finality_retry_failure_class_mismatch_for_failed_reason")
            if (
                isinstance(finality_retry_attempts_used, int)
                and isinstance(finality_retry_max_attempts, int)
                and finality_retry_attempts_used != finality_retry_max_attempts
            ):
                reason_codes.append("finality_retry_attempts_used_mismatch_for_failed_reason")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    for required_reason_code in args.require_reason_code:
        if reason_code != required_reason_code:
            reason_codes.append(f"required_reason_code_missing:{required_reason_code}")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
    elif status == "fail":
        observed_final_decision = "NO-GO"

    if observed_final_decision and observed_final_decision != args.expected_final_decision:
        reason_codes.append("observed_final_decision_mismatch")

    final_decision = "GO" if not reason_codes else "NO-GO"
    return final_decision, reason_codes


def main() -> int:
    args = parse_args()
    report_path = Path(args.report_file).resolve()
    report = json.loads(report_path.read_text(encoding="utf-8"))

    observed_status = report.get("status")
    observed_final_decision = ""
    if observed_status == "ok":
        observed_final_decision = "GO"
    elif observed_status == "fail":
        observed_final_decision = "NO-GO"

    final_decision, reason_codes = evaluate(report, args)
    output = {
        "schema_version": "kamn.kolme.local-runtime-commit-live-policy-report.v1",
        "report_file": str(report_path),
        "expected_final_decision": args.expected_final_decision,
        "ci_fast_gate": args.ci_fast_gate,
        "required_reason_codes": args.require_reason_code,
        "expected_provider_client_contract": args.expected_provider_client_contract,
        "require_non_synthetic_run_evidence": args.require_non_synthetic_run_evidence,
        "require_native_payload_evidence": args.require_native_payload_evidence,
        "observed_status": observed_status,
        "observed_provider_contract_enforcement_mode": report.get("provider_contract_enforcement_mode"),
        "observed_provider_live_contract_marker": report.get("provider_live_contract_marker"),
        "observed_provider_live_contract_marker_present": report.get(
            "provider_live_contract_marker_present"
        ),
        "observed_provider_in_memory_reference_detected": report.get(
            "provider_in_memory_reference_detected"
        ),
        "observed_final_decision": observed_final_decision,
        "observed_reason_code": report.get("reason_code"),
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    if args.output_json:
        output_path = Path(args.output_json).resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(output, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    status = "ok" if final_decision == "GO" else "fail"
    failed_checks = ",".join(reason_codes) if reason_codes else "none"
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == "GO" else 1


if __name__ == "__main__":
    raise SystemExit(main())
