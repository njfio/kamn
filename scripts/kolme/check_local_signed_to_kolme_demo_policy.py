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

    checks = report.get("checks")
    if not isinstance(checks, list) or not checks:
        reason_codes.append("checks_missing")
    else:
        expected_ids = {
            "localhost_signed_demo_contract",
            "localhost_signed_integration_contract",
            "local_kamn_runtime_integration_contract",
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
        missing_ids = sorted(expected_ids - observed_ids)
        for missing_id in missing_ids:
            reason_codes.append(f"check_missing:{missing_id}")

    artifacts = report.get("artifact_paths")
    if not isinstance(artifacts, list) or len(artifacts) < 4:
        reason_codes.append("artifact_paths_missing")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
        if reason_code not in ("dry_run_no_commands_executed", "signed_to_kolme_demo_passed"):
            reason_codes.append("ok_status_reason_code_mismatch")
        if budget_status == "exceeded_budget":
            reason_codes.append("ok_status_budget_exceeded")
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
