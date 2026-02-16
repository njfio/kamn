#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate unified local signed-to-Kolme demo summary policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(report: dict[str, object], args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []

    if report.get("schema_version") != "kamn.kolme.local-signed-to-kolme-demo-summary.v1":
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

    runtime_commit_submit_evidence_marker = report.get("runtime_commit_submit_evidence_marker")
    if runtime_commit_submit_evidence_marker != "status=submitted":
        reason_codes.append("runtime_commit_submit_evidence_marker_mismatch")

    runtime_commit_submit_evidence_marker_present = report.get(
        "runtime_commit_submit_evidence_marker_present"
    )
    if not isinstance(runtime_commit_submit_evidence_marker_present, bool):
        reason_codes.append("runtime_commit_submit_evidence_marker_present_invalid")

    runtime_commit_finality_evidence_marker = report.get("runtime_commit_finality_evidence_marker")
    if runtime_commit_finality_evidence_marker != "finality=final":
        reason_codes.append("runtime_commit_finality_evidence_marker_mismatch")

    runtime_commit_finality_evidence_marker_present = report.get(
        "runtime_commit_finality_evidence_marker_present"
    )
    if not isinstance(runtime_commit_finality_evidence_marker_present, bool):
        reason_codes.append("runtime_commit_finality_evidence_marker_present_invalid")

    runtime_commit_submit_finality_contract_version = report.get(
        "runtime_commit_submit_finality_contract_version"
    )
    if runtime_commit_submit_finality_contract_version != "v1":
        reason_codes.append("runtime_commit_submit_finality_contract_version_mismatch")

    runtime_commit_submit_finality_linked = report.get("runtime_commit_submit_finality_linked")
    if not isinstance(runtime_commit_submit_finality_linked, bool):
        reason_codes.append("runtime_commit_submit_finality_linked_invalid")

    runtime_commit_live_status = report.get("runtime_commit_live_status")
    if not isinstance(runtime_commit_live_status, str) or not runtime_commit_live_status.strip():
        reason_codes.append("runtime_commit_live_status_missing")

    runtime_commit_live_reason_code = report.get("runtime_commit_live_reason_code")
    if not isinstance(runtime_commit_live_reason_code, str) or not runtime_commit_live_reason_code.strip():
        reason_codes.append("runtime_commit_live_reason_code_missing")

    runtime_commit_live_summary_path = report.get("runtime_commit_live_summary_path")
    if not isinstance(runtime_commit_live_summary_path, str) or not runtime_commit_live_summary_path.strip():
        reason_codes.append("runtime_commit_live_summary_path_missing")

    runtime_commit_live_policy_report_path = report.get("runtime_commit_live_policy_report_path")
    if not isinstance(runtime_commit_live_policy_report_path, str) or not runtime_commit_live_policy_report_path.strip():
        reason_codes.append("runtime_commit_live_policy_report_path_missing")

    checks = report.get("checks")
    check_status_by_id: dict[str, str] = {}
    if not isinstance(checks, list) or not checks:
        reason_codes.append("checks_missing")
    else:
        expected_ids = {
            "localhost_signed_demo_contract",
            "localhost_signed_integration_contract",
            "local_kamn_runtime_integration_run",
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
            if not isinstance(command, str) or not command.strip():
                reason_codes.append(f"check_command_invalid:{check_id}")
            if check_status not in ("planned", "pass", "fail", "skipped"):
                reason_codes.append(f"check_status_invalid:{check_id}")
            if not isinstance(check_reason_code, str) or not check_reason_code.strip():
                reason_codes.append(f"check_reason_code_invalid:{check_id}")
            if check_id not in check_status_by_id and check_status in ("planned", "pass", "fail", "skipped"):
                check_status_by_id[check_id] = str(check_status)
        missing_ids = sorted(expected_ids - observed_ids)
        for missing_id in missing_ids:
            reason_codes.append(f"check_missing:{missing_id}")

    artifacts = report.get("artifact_paths")
    if not isinstance(artifacts, list) or len(artifacts) < 8:
        reason_codes.append("artifact_paths_missing")
    else:
        if (
            isinstance(runtime_commit_live_summary_path, str)
            and runtime_commit_live_summary_path.strip()
            and runtime_commit_live_summary_path not in artifacts
        ):
            reason_codes.append("runtime_commit_live_summary_artifact_path_missing")
        if (
            isinstance(runtime_commit_live_policy_report_path, str)
            and runtime_commit_live_policy_report_path.strip()
            and runtime_commit_live_policy_report_path not in artifacts
        ):
            reason_codes.append("runtime_commit_live_policy_artifact_path_missing")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
        if reason_code not in ("dry_run_no_commands_executed", "signed_to_kolme_demo_passed"):
            reason_codes.append("ok_status_reason_code_mismatch")
        if budget_status == "exceeded_budget":
            reason_codes.append("ok_status_budget_exceeded")
        if mode == "dry-run":
            if runtime_commit_submit_evidence_marker_present is not False:
                reason_codes.append("runtime_commit_submit_evidence_marker_unexpected_in_dry_run")
            if runtime_commit_finality_evidence_marker_present is not False:
                reason_codes.append("runtime_commit_finality_evidence_marker_unexpected_in_dry_run")
            if runtime_commit_submit_finality_linked is not False:
                reason_codes.append("runtime_commit_submit_finality_linkage_unexpected_in_dry_run")
        elif mode == "run":
            if runtime_commit_live_status != "ok":
                reason_codes.append("runtime_commit_live_status_mismatch")
            if runtime_commit_submit_evidence_marker_present is not True:
                reason_codes.append("runtime_commit_submit_evidence_marker_missing")
            if runtime_commit_finality_evidence_marker_present is not True:
                reason_codes.append("runtime_commit_finality_evidence_marker_missing")
            if runtime_commit_submit_finality_linked is not True:
                reason_codes.append("runtime_commit_submit_finality_linkage_missing")
            expected_checkpoint_ids = [
                "localhost_signed_demo_contract",
                "localhost_signed_integration_contract",
                "local_kamn_runtime_integration_run",
            ]
            if (
                runtime_commit_live_status == "ok"
                and runtime_commit_submit_evidence_marker_present is True
                and runtime_commit_finality_evidence_marker_present is True
                and runtime_commit_submit_finality_linked is True
                and all(check_id in check_status_by_id for check_id in expected_checkpoint_ids)
            ):
                mismatched_checkpoint_ids = [
                    check_id
                    for check_id in expected_checkpoint_ids
                    if check_status_by_id.get(check_id) != "pass"
                ]
                if mismatched_checkpoint_ids:
                    reason_codes.append("signed_message_commit_evidence_mismatch")
                    for check_id in sorted(mismatched_checkpoint_ids):
                        reason_codes.append(f"signed_message_commit_evidence_mismatch:{check_id}")
    elif status == "fail":
        observed_final_decision = "NO-GO"
        if reason_code in ("dry_run_no_commands_executed", "signed_to_kolme_demo_passed"):
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
        "schema_version": "kamn.kolme.local-signed-to-kolme-demo-policy-report.v1",
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
