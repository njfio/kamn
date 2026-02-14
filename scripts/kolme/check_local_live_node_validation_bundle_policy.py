#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate local live-node validation bundle summary policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(report: dict[str, object], args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []
    required_signing_profile_marker = "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1"
    expected_run_mode_reason_codes = {
        "integration_bundle": "integration_bundle_passed",
        "integration_policy": "integration_policy_passed",
        "process_lifecycle_bundle": "process_lifecycle_bundle_passed",
        "process_lifecycle_policy": "process_lifecycle_policy_passed",
    }

    if report.get("schema_version") != "kamn.kolme.local-live-node-validation-bundle-summary.v1":
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

    ci_fast_gate_eligible = report.get("ci_fast_gate_eligible")
    if not isinstance(ci_fast_gate_eligible, bool):
        reason_codes.append("ci_fast_gate_eligible_invalid")
    elif ci_fast_gate_eligible:
        reason_codes.append("ci_fast_gate_eligibility_violation")

    elapsed_seconds = report.get("elapsed_seconds")
    if not isinstance(elapsed_seconds, int) or elapsed_seconds < 0:
        reason_codes.append("elapsed_seconds_invalid")

    max_seconds = report.get("max_seconds")
    if not isinstance(max_seconds, int) or max_seconds <= 0:
        reason_codes.append("max_seconds_invalid")

    budget_status = report.get("budget_status")
    if budget_status not in ("not_run", "within_budget", "exceeded_budget"):
        reason_codes.append("budget_status_invalid")

    if not isinstance(report.get("checkout_path"), str) or not report["checkout_path"].strip():
        reason_codes.append("checkout_path_missing")
    if not isinstance(report.get("expected_remote_url"), str) or not report["expected_remote_url"].strip():
        reason_codes.append("expected_remote_url_missing")
    if not isinstance(report.get("expected_ref"), str) or not report["expected_ref"].strip():
        reason_codes.append("expected_ref_missing")
    if not isinstance(report.get("base_url"), str) or not report["base_url"].strip():
        reason_codes.append("base_url_missing")
    if not isinstance(report.get("fork_chain_version"), str) or not report["fork_chain_version"].strip():
        reason_codes.append("fork_chain_version_missing")

    integration_command = report.get("integration_command")
    if not isinstance(integration_command, str) or not integration_command.strip():
        reason_codes.append("integration_command_missing")
    else:
        if "run_local_kamn_live_runtime_integration_lane.sh" not in integration_command:
            reason_codes.append("integration_runner_missing")
        if "--runtime-provider-client-contract KolmeRuntimeCommitLiveProvider" not in integration_command:
            reason_codes.append("runtime_provider_contract_marker_missing")
        if "--runtime-profile real-node" not in integration_command:
            reason_codes.append("integration_runtime_profile_marker_missing")
        if required_signing_profile_marker not in integration_command:
            reason_codes.append("integration_signing_profile_marker_missing")
        if "KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated" in integration_command:
            reason_codes.append("integration_simulated_signing_profile_detected")

    integration_policy_command = report.get("integration_policy_command")
    if not isinstance(integration_policy_command, str) or not integration_policy_command.strip():
        reason_codes.append("integration_policy_command_missing")
    elif "check_local_kamn_live_runtime_integration_policy.py" not in integration_policy_command:
        reason_codes.append("integration_policy_command_marker_missing")

    process_command = report.get("process_lifecycle_command")
    if not isinstance(process_command, str) or not process_command.strip():
        reason_codes.append("process_lifecycle_command_missing")
    else:
        if "run_local_kolme_fork_process_lifecycle_lane.sh" not in process_command:
            reason_codes.append("process_lifecycle_runner_missing")
        if "--integration-runtime-commit-live-policy-report" not in process_command:
            reason_codes.append("process_lifecycle_policy_report_marker_missing")
        if "--rollback-evidence-file" not in process_command:
            reason_codes.append("process_lifecycle_rollback_marker_missing")
        if "--recovery-evidence-file" not in process_command:
            reason_codes.append("process_lifecycle_recovery_marker_missing")

    process_policy_command = report.get("process_lifecycle_policy_command")
    if not isinstance(process_policy_command, str) or not process_policy_command.strip():
        reason_codes.append("process_lifecycle_policy_command_missing")
    elif "check_local_kolme_fork_process_lifecycle_policy.py" not in process_policy_command:
        reason_codes.append("process_lifecycle_policy_command_marker_missing")

    rollback_evidence_file = report.get("rollback_evidence_file")
    if not isinstance(rollback_evidence_file, str) or not rollback_evidence_file.strip():
        reason_codes.append("rollback_evidence_file_missing")
        rollback_evidence_file = ""

    recovery_evidence_file = report.get("recovery_evidence_file")
    if not isinstance(recovery_evidence_file, str) or not recovery_evidence_file.strip():
        reason_codes.append("recovery_evidence_file_missing")
        recovery_evidence_file = ""

    if isinstance(process_command, str) and process_command.strip():
        if rollback_evidence_file and f"--rollback-evidence-file {rollback_evidence_file}" not in process_command:
            reason_codes.append("process_lifecycle_rollback_marker_path_mismatch")
        if recovery_evidence_file and f"--recovery-evidence-file {recovery_evidence_file}" not in process_command:
            reason_codes.append("process_lifecycle_recovery_marker_path_mismatch")

    required_report_paths = [
        ("integration_report", report.get("integration_report")),
        ("integration_policy_report", report.get("integration_policy_report")),
        ("integration_runtime_policy_report", report.get("integration_runtime_policy_report")),
        ("integration_runtime_commit_live_summary", report.get("integration_runtime_commit_live_summary")),
        ("process_lifecycle_report", report.get("process_lifecycle_report")),
        ("process_lifecycle_policy_report", report.get("process_lifecycle_policy_report")),
    ]
    for field_name, value in required_report_paths:
        if not isinstance(value, str) or not value.strip():
            reason_codes.append(f"{field_name}_missing")

    contracts = report.get("contracts")
    if not isinstance(contracts, dict):
        reason_codes.append("contracts_missing")
    else:
        if contracts.get("ci_fast_gate_scope") != "local-only":
            reason_codes.append("ci_fast_gate_scope_mismatch")
        if contracts.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
            reason_codes.append("runtime_provider_client_contract_contract_mismatch")
        if contracts.get("bundle_contract") != "live_node_release_bundle_v1":
            reason_codes.append("bundle_contract_mismatch")
        if contracts.get("live_run_rehearsal_lineage_required") is not True:
            reason_codes.append("live_run_rehearsal_lineage_required_contract_mismatch")
        if contracts.get("rollback_recovery_artifact_lineage_required") is not True:
            reason_codes.append("rollback_recovery_artifact_lineage_required_contract_mismatch")
        if contracts.get("process_lifecycle_rollback_evidence_option") != "--rollback-evidence-file":
            reason_codes.append("process_lifecycle_rollback_evidence_option_contract_mismatch")
        if contracts.get("process_lifecycle_recovery_evidence_option") != "--recovery-evidence-file":
            reason_codes.append("process_lifecycle_recovery_evidence_option_contract_mismatch")

    check_entries_by_id: dict[str, dict[str, object]] = {}
    checks = report.get("checks")
    if not isinstance(checks, list) or not checks:
        reason_codes.append("checks_missing")
    else:
        expected_ids = {
            "integration_bundle",
            "integration_policy",
            "process_lifecycle_bundle",
            "process_lifecycle_policy",
        }
        observed_ids: set[str] = set()
        for entry in checks:
            if not isinstance(entry, dict):
                reason_codes.append("check_entry_invalid")
                continue
            check_id = entry.get("id")
            command = entry.get("command")
            check_status = entry.get("status")
            check_reason_code = entry.get("reason_code")
            if not isinstance(check_id, str) or not check_id.strip():
                reason_codes.append("check_id_invalid")
                continue
            observed_ids.add(check_id)
            check_entries_by_id[check_id] = entry
            if not isinstance(command, str) or not command.strip():
                reason_codes.append(f"check_command_invalid:{check_id}")
            if check_status not in ("planned", "pass", "fail", "skipped"):
                reason_codes.append(f"check_status_invalid:{check_id}")
            if not isinstance(check_reason_code, str) or not check_reason_code.strip():
                reason_codes.append(f"check_reason_code_invalid:{check_id}")
            if (
                check_id == "integration_bundle"
                and isinstance(command, str)
                and "--runtime-provider-client-contract KolmeRuntimeCommitLiveProvider" not in command
            ):
                reason_codes.append("integration_bundle_provider_marker_missing")
            if (
                check_id == "integration_bundle"
                and isinstance(command, str)
                and "--runtime-profile real-node" not in command
            ):
                reason_codes.append("integration_bundle_runtime_profile_marker_missing")
            if (
                check_id == "integration_bundle"
                and isinstance(command, str)
                and required_signing_profile_marker not in command
            ):
                reason_codes.append("integration_bundle_signing_profile_marker_missing")
            if (
                check_id == "integration_bundle"
                and isinstance(command, str)
                and "KAMN_KOLME_LIVE_SIGNING_PROFILE=simulated" in command
            ):
                reason_codes.append("integration_bundle_simulated_signing_profile_detected")
            if (
                check_id == "process_lifecycle_bundle"
                and isinstance(command, str)
                and "--integration-runtime-commit-live-policy-report" not in command
            ):
                reason_codes.append("process_lifecycle_bundle_policy_report_marker_missing")
            if (
                check_id == "process_lifecycle_bundle"
                and isinstance(command, str)
                and "--rollback-evidence-file" not in command
            ):
                reason_codes.append("process_lifecycle_bundle_rollback_marker_missing")
            if (
                check_id == "process_lifecycle_bundle"
                and isinstance(command, str)
                and "--recovery-evidence-file" not in command
            ):
                reason_codes.append("process_lifecycle_bundle_recovery_marker_missing")
            if (
                check_id == "process_lifecycle_bundle"
                and isinstance(command, str)
                and rollback_evidence_file
                and f"--rollback-evidence-file {rollback_evidence_file}" not in command
            ):
                reason_codes.append("process_lifecycle_bundle_rollback_marker_path_mismatch")
            if (
                check_id == "process_lifecycle_bundle"
                and isinstance(command, str)
                and recovery_evidence_file
                and f"--recovery-evidence-file {recovery_evidence_file}" not in command
            ):
                reason_codes.append("process_lifecycle_bundle_recovery_marker_path_mismatch")
        missing_ids = sorted(expected_ids - observed_ids)
        for missing_id in missing_ids:
            reason_codes.append(f"check_missing:{missing_id}")

    if mode == "run" and status == "ok":
        for check_id, expected_reason_code in expected_run_mode_reason_codes.items():
            entry = check_entries_by_id.get(check_id)
            if not isinstance(entry, dict):
                continue
            if entry.get("status") != "pass":
                reason_codes.append(f"run_mode_check_status_mismatch:{check_id}")
            if entry.get("reason_code") != expected_reason_code:
                reason_codes.append(f"run_mode_check_reason_code_mismatch:{check_id}")

    required_artifact_paths = list(required_report_paths)
    required_artifact_paths.extend(
        [
            ("rollback_evidence_file", rollback_evidence_file),
            ("recovery_evidence_file", recovery_evidence_file),
        ]
    )
    artifact_paths = report.get("artifact_paths")
    if not isinstance(artifact_paths, list) or not artifact_paths:
        reason_codes.append("artifact_paths_missing")
    else:
        for field_name, value in required_artifact_paths:
            if isinstance(value, str) and value.strip() and value not in artifact_paths:
                reason_codes.append(f"{field_name}_artifact_missing")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
        if reason_code not in ("dry_run_no_commands_executed", "live_node_validation_bundle_passed"):
            reason_codes.append("ok_status_reason_code_mismatch")
        if mode == "run" and budget_status != "within_budget":
            reason_codes.append("run_mode_budget_status_mismatch")
        if budget_status == "exceeded_budget":
            reason_codes.append("ok_status_budget_exceeded")
    elif status == "fail":
        observed_final_decision = "NO-GO"
        if reason_code in ("dry_run_no_commands_executed", "live_node_validation_bundle_passed"):
            reason_codes.append("fail_status_reason_code_mismatch")

    for required_reason_code in args.require_reason_code:
        if reason_code != required_reason_code:
            reason_codes.append(f"required_reason_code_missing:{required_reason_code}")

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
        "schema_version": "kamn.kolme.local-live-node-validation-bundle-policy-report.v1",
        "report_file": str(report_path),
        "expected_final_decision": args.expected_final_decision,
        "ci_fast_gate": args.ci_fast_gate,
        "required_reason_codes": args.require_reason_code,
        "observed_status": observed_status,
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
