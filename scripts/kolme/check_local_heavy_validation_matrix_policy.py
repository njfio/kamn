#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

EXPECTED_COMMAND_SNIPPETS = [
    "run_local_bootstrap_health_checks.sh",
    "run_version_compatibility_replay_deep_lane.sh",
    "run_local_kolme_fork_rust_test_matrix_contract_lane.sh",
    "run_local_kolme_live_api_conformance_contract_lane.sh",
]

EXPECTED_ARTIFACT_SNIPPETS = [
    "/tmp/kolme-local-bootstrap-summary.json",
    "/tmp/kolme-version-compatibility-report.json",
    "/tmp/kolme-local-fork-rust-test-matrix-summary.json",
    "/tmp/kolme-local-fork-rust-test-matrix-policy.json",
    "/tmp/kolme-local-live-api-conformance-summary.json",
    "/tmp/kolme-local-live-api-conformance-policy.json",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate local heavy validation matrix summary policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(report: dict[str, object], args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []

    if report.get("schema_version") != "kamn.kolme.local-heavy-validation-summary.v1":
        reason_codes.append("schema_version_mismatch")

    if report.get("summary_type") != "commands":
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

    commands = report.get("commands")
    if not isinstance(commands, list) or not commands:
        reason_codes.append("commands_missing")
        commands = []
    if isinstance(commands, list):
        if not all(isinstance(command, str) and command.strip() for command in commands):
            reason_codes.append("commands_invalid")
        for expected_snippet in EXPECTED_COMMAND_SNIPPETS:
            if not any(expected_snippet in command for command in commands if isinstance(command, str)):
                reason_codes.append(f"command_missing:{expected_snippet}")

    artifact_paths = report.get("artifact_paths")
    if not isinstance(artifact_paths, list) or not artifact_paths:
        reason_codes.append("artifact_paths_missing")
        artifact_paths = []
    if isinstance(artifact_paths, list):
        if not all(isinstance(path, str) and path.strip() for path in artifact_paths):
            reason_codes.append("artifact_paths_invalid")
        for expected_snippet in EXPECTED_ARTIFACT_SNIPPETS:
            if not any(expected_snippet in path for path in artifact_paths if isinstance(path, str)):
                reason_codes.append(f"artifact_missing:{expected_snippet}")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
        if mode == "dry-run" and reason_code != "dry_run_no_commands_executed":
            reason_codes.append("dry_run_reason_code_mismatch")
        if mode == "run" and reason_code != "local_heavy_validation_passed":
            reason_codes.append("run_ok_reason_code_mismatch")
    elif status == "fail":
        observed_final_decision = "NO-GO"
        if reason_code in ("dry_run_no_commands_executed", "local_heavy_validation_passed"):
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
        "schema_version": "kamn.kolme.local-heavy-validation-policy-report.v1",
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
