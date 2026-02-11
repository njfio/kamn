#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

EXPECTED_CHECKPOINT_ORDER = [
    "local_opt_in_guard",
    "mold_linker_probe",
    "kolme_compile_probe",
    "libudev_probe",
    "integration_compile_probe",
]

EXPECTED_COMMAND_SNIPPETS = {
    "local_opt_in_guard": "assert_local_heavy_opt_in.sh",
    "mold_linker_probe": "mold",
    "kolme_compile_probe": "cargo test -p kolme",
    "libudev_probe": "pkg-config --libs --cflags libudev",
    "integration_compile_probe": "cargo test -p integration-tests --test six-sigma",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate local Kolme fork portability preflight summary policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(report: dict[str, object], args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []

    if report.get("schema_version") != "kamn.kolme.local-fork-portability-preflight-summary.v1":
        reason_codes.append("schema_version_mismatch")

    if report.get("summary_type") != "checkpoints":
        reason_codes.append("summary_type_mismatch")

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
    if budget_status not in ("pass", "fail"):
        reason_codes.append("budget_status_invalid")

    checkpoints = report.get("checkpoints")
    observed_ids: list[str] = []
    if not isinstance(checkpoints, list) or not checkpoints:
        reason_codes.append("checkpoints_missing")
    else:
        for entry in checkpoints:
            if not isinstance(entry, dict):
                reason_codes.append("checkpoint_entry_invalid")
                continue

            checkpoint_id = entry.get("id")
            checkpoint_command = entry.get("command")
            checkpoint_status = entry.get("status")

            if not isinstance(checkpoint_id, str) or not checkpoint_id.strip():
                reason_codes.append("checkpoint_id_invalid")
                continue

            observed_ids.append(checkpoint_id)

            if not isinstance(checkpoint_command, str) or not checkpoint_command.strip():
                reason_codes.append(f"checkpoint_command_invalid:{checkpoint_id}")
            else:
                expected_command_snippet = EXPECTED_COMMAND_SNIPPETS.get(checkpoint_id)
                if expected_command_snippet and expected_command_snippet not in checkpoint_command:
                    reason_codes.append(f"checkpoint_command_mismatch:{checkpoint_id}")

            if checkpoint_status not in ("planned", "pass", "fail", "skipped"):
                reason_codes.append(f"checkpoint_status_invalid:{checkpoint_id}")
            elif mode == "dry-run" and checkpoint_status != "planned":
                reason_codes.append(f"checkpoint_status_invalid_dry_run:{checkpoint_id}")
            elif mode == "run" and status == "ok" and checkpoint_status != "pass":
                reason_codes.append(f"checkpoint_status_invalid_run_ok:{checkpoint_id}")

        expected_set = set(EXPECTED_CHECKPOINT_ORDER)
        observed_set = set(observed_ids)
        for missing_id in sorted(expected_set - observed_set):
            reason_codes.append(f"check_missing:{missing_id}")
        for unknown_id in sorted(observed_set - expected_set):
            reason_codes.append(f"check_unknown:{unknown_id}")
        if observed_ids and observed_ids != EXPECTED_CHECKPOINT_ORDER:
            reason_codes.append("checkpoint_order_mismatch")

    artifact_paths = report.get("artifact_paths")
    if not isinstance(artifact_paths, list) or len(artifact_paths) < 4:
        reason_codes.append("artifact_paths_missing")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
        if mode == "dry-run" and reason_code != "dry_run_no_commands_executed":
            reason_codes.append("dry_run_reason_code_mismatch")
        if mode == "run" and reason_code != "portability_preflight_passed":
            reason_codes.append("run_ok_reason_code_mismatch")
        if budget_status == "fail":
            reason_codes.append("ok_status_budget_failed")
    elif status == "fail":
        observed_final_decision = "NO-GO"
        if reason_code in ("dry_run_no_commands_executed", "portability_preflight_passed"):
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
        "schema_version": "kamn.kolme.local-fork-portability-preflight-policy-report.v1",
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
