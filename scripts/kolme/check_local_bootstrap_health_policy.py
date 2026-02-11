#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

EXPECTED_CHECK_ORDER = [
    "version_compatibility",
    "fork_compatibility_evidence",
    "fork_compatibility_policy",
    "triadic_devnet_smoke",
    "triadic_devnet_validate",
]

EXPECTED_COMMAND_SNIPPETS = {
    "version_compatibility": "validate_version_compatibility.py",
    "fork_compatibility_evidence": "generate_fork_compatibility_evidence.py",
    "fork_compatibility_policy": "check_fork_compatibility_policy.py",
    "triadic_devnet_smoke": "run_triadic_devnet_smoke.sh",
    "triadic_devnet_validate": "validate_triadic_devnet_smoke.py",
}

EXPECTED_ARTIFACT_SNIPPETS = [
    "/tmp/kolme-bootstrap-version-report.json",
    "/tmp/kolme-bootstrap-fork-compatibility-report.json",
    "/tmp/kolme-bootstrap-fork-compatibility-policy-report.json",
    "/tmp/kolme-bootstrap-devnet-markers.txt",
    "/tmp/kolme-bootstrap-devnet-report.json",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate local bootstrap health summary policy."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(report: dict[str, object], args: argparse.Namespace) -> tuple[str, list[str]]:
    reason_codes: list[str] = []

    if report.get("schema_version") != "kamn.kolme.local-bootstrap-summary.v1":
        reason_codes.append("schema_version_mismatch")

    mode = report.get("mode")
    if mode not in ("dry-run", "run"):
        reason_codes.append("mode_invalid")

    status = report.get("status")
    if status not in ("ok", "fail"):
        reason_codes.append("status_invalid")

    ready = report.get("ready")
    if not isinstance(ready, bool):
        reason_codes.append("ready_invalid")

    readiness_status = report.get("readiness_status")
    if readiness_status not in ("planned", "ready", "failed"):
        reason_codes.append("readiness_status_invalid")

    reason_code = report.get("reason_code")
    if not isinstance(reason_code, str) or not reason_code.strip():
        reason_codes.append("reason_code_missing")

    if report.get("local_only_enforced") is not True:
        reason_codes.append("local_only_enforced_missing")

    checks = report.get("checks")
    observed_ids: list[str] = []
    if not isinstance(checks, list) or not checks:
        reason_codes.append("checks_missing")
        checks = []
    if isinstance(checks, list):
        for entry in checks:
            if not isinstance(entry, dict):
                reason_codes.append("check_entry_invalid")
                continue

            check_id = entry.get("id")
            check_command = entry.get("command")
            check_status = entry.get("status")

            if not isinstance(check_id, str) or not check_id.strip():
                reason_codes.append("check_id_invalid")
                continue
            observed_ids.append(check_id)

            if not isinstance(check_command, str) or not check_command.strip():
                reason_codes.append(f"check_command_invalid:{check_id}")
            else:
                expected_command_snippet = EXPECTED_COMMAND_SNIPPETS.get(check_id)
                if expected_command_snippet and expected_command_snippet not in check_command:
                    reason_codes.append(f"check_command_mismatch:{check_id}")

            if check_status not in ("planned", "pass", "fail", "skipped"):
                reason_codes.append(f"check_status_invalid:{check_id}")
            elif mode == "dry-run" and check_status != "planned":
                reason_codes.append(f"check_status_invalid_dry_run:{check_id}")
            elif mode == "run" and status == "ok" and check_status != "pass":
                reason_codes.append(f"check_status_invalid_run_ok:{check_id}")

        expected_set = set(EXPECTED_CHECK_ORDER)
        observed_set = set(observed_ids)
        for missing_id in sorted(expected_set - observed_set):
            reason_codes.append(f"check_missing:{missing_id}")
        for unknown_id in sorted(observed_set - expected_set):
            reason_codes.append(f"check_unknown:{unknown_id}")
        if observed_ids and observed_ids != EXPECTED_CHECK_ORDER:
            reason_codes.append("check_order_mismatch")

    artifact_paths = report.get("artifact_paths")
    if not isinstance(artifact_paths, list) or not artifact_paths:
        reason_codes.append("artifact_paths_missing")
        artifact_paths = []
    if isinstance(artifact_paths, list):
        if not all(isinstance(path, str) and path.strip() for path in artifact_paths):
            reason_codes.append("artifact_paths_invalid")
        for expected_artifact_snippet in EXPECTED_ARTIFACT_SNIPPETS:
            if not any(
                expected_artifact_snippet in path
                for path in artifact_paths
                if isinstance(path, str)
            ):
                reason_codes.append(f"artifact_missing:{expected_artifact_snippet}")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    observed_final_decision = ""
    if status == "ok":
        observed_final_decision = "GO"
        if mode == "dry-run":
            if reason_code != "dry_run_no_commands_executed":
                reason_codes.append("dry_run_reason_code_mismatch")
            if readiness_status != "planned":
                reason_codes.append("dry_run_readiness_status_mismatch")
            if ready is not False:
                reason_codes.append("dry_run_ready_mismatch")
        if mode == "run":
            if reason_code != "local_bootstrap_health_checks_passed":
                reason_codes.append("run_ok_reason_code_mismatch")
            if readiness_status != "ready":
                reason_codes.append("run_ok_readiness_status_mismatch")
            if ready is not True:
                reason_codes.append("run_ok_ready_mismatch")
    elif status == "fail":
        observed_final_decision = "NO-GO"
        if isinstance(reason_code, str) and not reason_code.startswith("bootstrap_check_failed_"):
            reason_codes.append("fail_reason_code_mismatch")
        if readiness_status != "failed":
            reason_codes.append("fail_readiness_status_mismatch")
        if ready is not False:
            reason_codes.append("fail_ready_mismatch")

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
        "schema_version": "kamn.kolme.local-bootstrap-policy-report.v1",
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
