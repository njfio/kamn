#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

EXPECTED_PRIMARY_CHECK_ORDER = [
    "process_start",
    "readiness_probe",
    "kamn_live_integration",
    "process_teardown",
    "rollback_evidence",
    "recovery_evidence",
]


def classify_overall_reason(status_value: str, reason_value: str) -> str:
    if status_value == "ok" and reason_value == "dry_run_no_commands_executed":
        return "lifecycle.not_run"
    if status_value == "ok":
        return "lifecycle.success"
    if reason_value in (
        "local_opt_in_missing",
        "serve_command_missing",
        "process_start_failed",
        "process_readiness_failed",
    ):
        return "lifecycle.startup_failed"
    if reason_value == "kamn_live_integration_failed":
        return "lifecycle.integration_failed"
    if reason_value == "process_teardown_failed":
        return "lifecycle.teardown_failed"
    if reason_value == "process_lifecycle_budget_exceeded":
        return "lifecycle.budget_exceeded"
    return "lifecycle.failed"


def classify_start_reason(reason_value: str) -> str:
    mapping = {
        "not_run": "startup.not_run",
        "process_started": "startup.process_started",
        "local_opt_in_missing": "startup.local_opt_in_missing",
        "serve_command_missing": "startup.serve_command_missing",
        "process_start_failed": "startup.process_start_failed",
    }
    return mapping.get(reason_value, "startup.other")


def classify_readiness_reason(reason_value: str) -> str:
    mapping = {
        "not_run": "readiness.not_run",
        "readiness_checks_passed": "readiness.checks_passed",
        "process_readiness_failed": "readiness.checks_failed",
        "local_opt_in_missing": "readiness.prerequisite_failed",
        "serve_command_missing": "readiness.prerequisite_failed",
        "process_start_failed": "readiness.prerequisite_failed",
    }
    return mapping.get(reason_value, "readiness.other")


def classify_integration_reason(reason_value: str) -> str:
    mapping = {
        "not_run": "integration.not_run",
        "kamn_live_integration_passed": "integration.passed",
        "kamn_live_integration_timeout": "integration.timeout",
        "process_readiness_failed": "integration.prerequisite_failed",
        "local_opt_in_missing": "integration.prerequisite_failed",
        "serve_command_missing": "integration.prerequisite_failed",
        "process_start_failed": "integration.prerequisite_failed",
    }
    return mapping.get(reason_value, "integration.failed")


def classify_teardown_reason(reason_value: str) -> str:
    mapping = {
        "not_run": "teardown.not_run",
        "process_teardown_passed": "teardown.passed",
        "process_teardown_forced": "teardown.forced",
        "local_opt_in_missing": "teardown.skipped_prerequisite_failed",
        "serve_command_missing": "teardown.skipped_prerequisite_failed",
        "process_start_failed": "teardown.skipped_prerequisite_failed",
    }
    return mapping.get(reason_value, "teardown.other")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate local Kolme fork process lifecycle summary policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(report: dict[str, object], args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []

    if report.get("schema_version") != "kamn.kolme.local-fork-process-lifecycle-summary.v1":
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

    elapsed_seconds = report.get("elapsed_seconds")
    if not isinstance(elapsed_seconds, int) or elapsed_seconds < 0:
        reason_codes.append("elapsed_seconds_invalid")

    max_seconds = report.get("max_seconds")
    if not isinstance(max_seconds, int) or max_seconds <= 0:
        reason_codes.append("max_seconds_invalid")

    budget_status = report.get("budget_status")
    if budget_status not in ("not_run", "within_budget", "exceeded_budget"):
        reason_codes.append("budget_status_invalid")

    contracts = report.get("contracts")
    if not isinstance(contracts, dict):
        reason_codes.append("contracts_missing")
    else:
        if contracts.get("healthz_path") != "/healthz":
            reason_codes.append("healthz_path_mismatch")
        if contracts.get("fork_info_path") != "/fork-info":
            reason_codes.append("fork_info_path_mismatch")
        if contracts.get("runtime_commit_endpoint") != "/broadcast/runtime-commit":
            reason_codes.append("runtime_commit_endpoint_mismatch")
        if contracts.get("runtime_commit_method") != "POST":
            reason_codes.append("runtime_commit_method_mismatch")
        if contracts.get("integration_runner") != "run_local_kamn_live_runtime_integration_lane.sh":
            reason_codes.append("integration_runner_mismatch")
        if contracts.get("runtime_provider_client_contract") != "KolmeRuntimeCommitLiveProvider":
            reason_codes.append("runtime_provider_client_contract_contract_mismatch")
        if contracts.get("integration_runtime_commit_live_policy_report_option") != "--runtime-commit-live-policy-report":
            reason_codes.append("integration_runtime_commit_live_policy_report_option_mismatch")
        if contracts.get("rollback_evidence_option") != "--rollback-evidence-file":
            reason_codes.append("rollback_evidence_option_mismatch")
        if contracts.get("recovery_evidence_option") != "--recovery-evidence-file":
            reason_codes.append("recovery_evidence_option_mismatch")
        if contracts.get("rollback_evidence_marker") != "kamn.kolme.local-fork-process-lifecycle.rollback-evidence.v1":
            reason_codes.append("rollback_evidence_marker_mismatch")
        if contracts.get("recovery_evidence_marker") != "kamn.kolme.local-fork-process-lifecycle.recovery-evidence.v1":
            reason_codes.append("recovery_evidence_marker_mismatch")

    integration_runtime_commit_live_policy_report = report.get("integration_runtime_commit_live_policy_report")
    if (
        not isinstance(integration_runtime_commit_live_policy_report, str)
        or not integration_runtime_commit_live_policy_report.strip()
    ):
        reason_codes.append("integration_runtime_commit_live_policy_report_missing")

    integration_runtime_commit_policy_reason_code = report.get("integration_runtime_commit_policy_reason_code")
    if (
        not isinstance(integration_runtime_commit_policy_reason_code, str)
        or not integration_runtime_commit_policy_reason_code.strip()
    ):
        reason_codes.append("integration_runtime_commit_policy_reason_code_missing")

    rollback_evidence_file = report.get("rollback_evidence_file")
    if not isinstance(rollback_evidence_file, str) or not rollback_evidence_file.strip():
        reason_codes.append("rollback_evidence_file_missing")
    recovery_evidence_file = report.get("recovery_evidence_file")
    if not isinstance(recovery_evidence_file, str) or not recovery_evidence_file.strip():
        reason_codes.append("recovery_evidence_file_missing")
    rollback_evidence_status = report.get("rollback_evidence_status")
    if rollback_evidence_status not in ("planned", "not_required", "required"):
        reason_codes.append("rollback_evidence_status_invalid")
    recovery_evidence_status = report.get("recovery_evidence_status")
    if recovery_evidence_status not in ("planned", "validated", "required"):
        reason_codes.append("recovery_evidence_status_invalid")
    rollback_evidence_reason_code = report.get("rollback_evidence_reason_code")
    if not isinstance(rollback_evidence_reason_code, str) or not rollback_evidence_reason_code.strip():
        reason_codes.append("rollback_evidence_reason_code_missing")
    recovery_evidence_reason_code = report.get("recovery_evidence_reason_code")
    if not isinstance(recovery_evidence_reason_code, str) or not recovery_evidence_reason_code.strip():
        reason_codes.append("recovery_evidence_reason_code_missing")
    start_reason_code = report.get("start_reason_code")
    if not isinstance(start_reason_code, str) or not start_reason_code.strip():
        reason_codes.append("start_reason_code_missing")
    readiness_reason_code = report.get("readiness_reason_code")
    if not isinstance(readiness_reason_code, str) or not readiness_reason_code.strip():
        reason_codes.append("readiness_reason_code_missing")
    integration_reason_code = report.get("integration_reason_code")
    if not isinstance(integration_reason_code, str) or not integration_reason_code.strip():
        reason_codes.append("integration_reason_code_missing")
    teardown_reason_code = report.get("teardown_reason_code")
    if not isinstance(teardown_reason_code, str) or not teardown_reason_code.strip():
        reason_codes.append("teardown_reason_code_missing")

    reason_taxonomy = report.get("reason_taxonomy")
    if not isinstance(reason_taxonomy, dict):
        reason_codes.append("reason_taxonomy_missing")
    else:
        if reason_taxonomy.get("schema_version") != "kamn.kolme.local-fork-process-lifecycle.reason-taxonomy.v1":
            reason_codes.append("reason_taxonomy_schema_mismatch")
        expected_overall_taxonomy = classify_overall_reason(
            str(status) if isinstance(status, str) else "",
            reason_code if isinstance(reason_code, str) else "",
        )
        if reason_taxonomy.get("overall") != expected_overall_taxonomy:
            reason_codes.append("reason_taxonomy_overall_mismatch")
        expected_start_taxonomy = classify_start_reason(
            start_reason_code if isinstance(start_reason_code, str) else ""
        )
        if reason_taxonomy.get("startup") != expected_start_taxonomy:
            reason_codes.append("reason_taxonomy_startup_mismatch")
        expected_readiness_taxonomy = classify_readiness_reason(
            readiness_reason_code if isinstance(readiness_reason_code, str) else ""
        )
        if reason_taxonomy.get("readiness") != expected_readiness_taxonomy:
            reason_codes.append("reason_taxonomy_readiness_mismatch")
        expected_integration_taxonomy = classify_integration_reason(
            integration_reason_code if isinstance(integration_reason_code, str) else ""
        )
        if reason_taxonomy.get("integration") != expected_integration_taxonomy:
            reason_codes.append("reason_taxonomy_integration_mismatch")
        expected_teardown_taxonomy = classify_teardown_reason(
            teardown_reason_code if isinstance(teardown_reason_code, str) else ""
        )
        if reason_taxonomy.get("teardown") != expected_teardown_taxonomy:
            reason_codes.append("reason_taxonomy_teardown_mismatch")

    normalized_evidence = report.get("normalized_evidence")
    normalized_checks_by_id: dict[str, object] = {}
    if not isinstance(normalized_evidence, dict):
        reason_codes.append("normalized_evidence_missing")
    else:
        if normalized_evidence.get("schema_version") != "kamn.kolme.local-fork-process-lifecycle.evidence-normalization.v1":
            reason_codes.append("normalized_evidence_schema_mismatch")
        if normalized_evidence.get("primary_check_order") != EXPECTED_PRIMARY_CHECK_ORDER:
            reason_codes.append("normalized_evidence_primary_check_order_mismatch")
        checks_by_id = normalized_evidence.get("checks_by_id")
        if not isinstance(checks_by_id, dict):
            reason_codes.append("normalized_evidence_checks_by_id_missing")
        else:
            normalized_checks_by_id = checks_by_id
            for check_id in EXPECTED_PRIMARY_CHECK_ORDER:
                check_entry = checks_by_id.get(check_id)
                if not isinstance(check_entry, dict):
                    reason_codes.append(f"normalized_evidence_check_missing:{check_id}")
                    continue
                if not isinstance(check_entry.get("status"), str) or not check_entry.get("status"):
                    reason_codes.append(f"normalized_evidence_status_invalid:{check_id}")
                if (
                    not isinstance(check_entry.get("reason_code"), str)
                    or not check_entry.get("reason_code")
                ):
                    reason_codes.append(f"normalized_evidence_reason_code_invalid:{check_id}")
                if not isinstance(check_entry.get("command"), str):
                    reason_codes.append(f"normalized_evidence_command_invalid:{check_id}")

    artifact_paths = report.get("artifact_paths")
    if not isinstance(artifact_paths, list) or not artifact_paths:
        reason_codes.append("artifact_paths_missing")
    elif (
        isinstance(integration_runtime_commit_live_policy_report, str)
        and integration_runtime_commit_live_policy_report.strip()
        and integration_runtime_commit_live_policy_report not in artifact_paths
    ):
        reason_codes.append("integration_runtime_commit_live_policy_report_artifact_missing")
    if (
        isinstance(artifact_paths, list)
        and isinstance(rollback_evidence_file, str)
        and rollback_evidence_file.strip()
        and rollback_evidence_file not in artifact_paths
    ):
        reason_codes.append("rollback_evidence_artifact_missing")
    if (
        isinstance(artifact_paths, list)
        and isinstance(recovery_evidence_file, str)
        and recovery_evidence_file.strip()
        and recovery_evidence_file not in artifact_paths
    ):
        reason_codes.append("recovery_evidence_artifact_missing")

    checks = report.get("checks")
    check_entries_by_id: dict[str, list[dict[str, str]]] = {}
    check_order: list[str] = []
    if not isinstance(checks, list) or not checks:
        reason_codes.append("checks_missing")
    else:
        expected_order = EXPECTED_PRIMARY_CHECK_ORDER
        expected_ids = set(expected_order)
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
            check_order.append(check_id)
            if not isinstance(command, str) or not command.strip():
                reason_codes.append(f"check_command_invalid:{check_id}")
            if check_status not in ("planned", "pass", "fail", "skipped"):
                reason_codes.append(f"check_status_invalid:{check_id}")
            if not isinstance(check_reason_code, str) or not check_reason_code.strip():
                reason_codes.append(f"check_reason_code_invalid:{check_id}")
            check_entries_by_id.setdefault(check_id, []).append(
                {
                    "status": str(check_status),
                    "reason_code": str(check_reason_code),
                    "command": str(command),
                }
            )
            if (
                check_id == "kamn_live_integration"
                and isinstance(command, str)
                and "--runtime-commit-live-policy-report" not in command
            ):
                reason_codes.append("kamn_live_integration_policy_report_marker_missing")
            if (
                check_id == "kamn_live_integration"
                and isinstance(command, str)
                and "--runtime-profile real-node" not in command
            ):
                reason_codes.append("kamn_live_integration_runtime_profile_marker_missing")
            if (
                check_id == "kamn_live_integration"
                and isinstance(command, str)
                and "--runtime-provider-client-contract KolmeRuntimeCommitLiveProvider" not in command
            ):
                reason_codes.append("kamn_live_integration_runtime_provider_marker_missing")
            if (
                check_id == "rollback_evidence"
                and isinstance(command, str)
                and "--rollback-evidence-file" not in command
            ):
                reason_codes.append("rollback_evidence_marker_missing")
            if (
                check_id == "recovery_evidence"
                and isinstance(command, str)
                and "--recovery-evidence-file" not in command
            ):
                reason_codes.append("recovery_evidence_marker_missing")
        missing_ids = sorted(expected_ids - observed_ids)
        for missing_id in missing_ids:
            reason_codes.append(f"check_missing:{missing_id}")

        if normalized_checks_by_id and expected_ids.issubset(set(check_entries_by_id.keys())):
            for check_id in expected_order:
                normalized_entry = normalized_checks_by_id.get(check_id)
                if not isinstance(normalized_entry, dict):
                    continue
                first_entry = check_entries_by_id[check_id][0]
                if normalized_entry.get("status") != first_entry["status"]:
                    reason_codes.append(f"normalized_evidence_status_mismatch:{check_id}")
                if normalized_entry.get("reason_code") != first_entry["reason_code"]:
                    reason_codes.append(f"normalized_evidence_reason_code_mismatch:{check_id}")
                if normalized_entry.get("command") != first_entry["command"]:
                    reason_codes.append(f"normalized_evidence_command_mismatch:{check_id}")

        for check_id, entries in check_entries_by_id.items():
            if len(entries) > 1:
                reason_codes.append(f"check_id_duplicate:{check_id}")

        observed_primary_order: list[str] = []
        seen_primary_ids: set[str] = set()
        for check_id in check_order:
            if check_id in expected_ids and check_id not in seen_primary_ids:
                observed_primary_order.append(check_id)
                seen_primary_ids.add(check_id)
        if len(observed_primary_order) == len(expected_order) and observed_primary_order != expected_order:
            reason_codes.append("check_sequence_mismatch")

        if expected_ids.issubset(set(check_entries_by_id.keys())):
            process_start_status = check_entries_by_id["process_start"][0]["status"]
            readiness_status = check_entries_by_id["readiness_probe"][0]["status"]
            integration_status = check_entries_by_id["kamn_live_integration"][0]["status"]
            teardown_status = check_entries_by_id["process_teardown"][0]["status"]

            if readiness_status == "pass" and process_start_status != "pass":
                reason_codes.append("startup_dependency_drift:readiness_without_process_start")
            if integration_status == "pass" and readiness_status != "pass":
                reason_codes.append("startup_dependency_drift:integration_without_readiness")
            if integration_status == "pass" and process_start_status != "pass":
                reason_codes.append("startup_dependency_drift:integration_without_process_start")
            if teardown_status == "pass" and integration_status not in ("pass", "fail"):
                reason_codes.append("startup_dependency_drift:teardown_without_integration_outcome")

            if mode == "dry-run":
                if start_reason_code != "not_run":
                    reason_codes.append("dry_run_start_reason_code_mismatch")
                if readiness_reason_code != "not_run":
                    reason_codes.append("dry_run_readiness_reason_code_mismatch")
                if integration_reason_code != "not_run":
                    reason_codes.append("dry_run_integration_reason_code_mismatch")
                if teardown_reason_code != "not_run":
                    reason_codes.append("dry_run_teardown_reason_code_mismatch")

            if mode == "run" and status == "ok":
                if process_start_status != "pass":
                    reason_codes.append("run_ok_process_start_status_mismatch")
                if readiness_status != "pass":
                    reason_codes.append("run_ok_readiness_status_mismatch")
                if integration_status != "pass":
                    reason_codes.append("run_ok_integration_status_mismatch")
                if teardown_status != "pass":
                    reason_codes.append("run_ok_teardown_status_mismatch")
                if start_reason_code != "process_started":
                    reason_codes.append("run_ok_start_reason_code_mismatch")
                if readiness_reason_code != "readiness_checks_passed":
                    reason_codes.append("run_ok_readiness_reason_code_mismatch")
                if integration_reason_code != "kamn_live_integration_passed":
                    reason_codes.append("run_ok_integration_reason_code_mismatch")
                if teardown_reason_code != "process_teardown_passed":
                    reason_codes.append("run_ok_teardown_reason_code_mismatch")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
        if reason_code not in ("dry_run_no_commands_executed", "process_lifecycle_integration_passed"):
            reason_codes.append("ok_status_reason_code_mismatch")
        if budget_status == "exceeded_budget":
            reason_codes.append("ok_status_budget_exceeded")
    elif status == "fail":
        observed_final_decision = "NO-GO"
        if reason_code in ("dry_run_no_commands_executed", "process_lifecycle_integration_passed"):
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
        "schema_version": "kamn.kolme.local-fork-process-lifecycle-policy-report.v1",
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
