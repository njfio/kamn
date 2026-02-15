#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

SCENARIO_MATRIX_SCHEMA_VERSION = "kamn.kolme.local-heavy-validation-scenario-matrix.v1"
EXPECTED_SCENARIO_MATRIX = [
    {
        "scenario_id": "bootstrap_health",
        "command_snippet": "run_local_bootstrap_health_checks.sh",
        "artifact_snippet": "/tmp/kolme-local-bootstrap-summary.json",
    },
    {
        "scenario_id": "version_compatibility_replay",
        "command_snippet": "run_version_compatibility_replay_deep_lane.sh",
        "artifact_snippet": "/tmp/kolme-version-compatibility-report.json",
    },
    {
        "scenario_id": "fork_rust_matrix",
        "command_snippet": "run_local_kolme_fork_rust_test_matrix_contract_lane.sh",
        "artifact_snippet": "/tmp/kolme-local-fork-rust-test-matrix-summary.json",
    },
    {
        "scenario_id": "live_api_conformance",
        "command_snippet": "run_local_kolme_live_api_conformance_contract_lane.sh",
        "artifact_snippet": "/tmp/kolme-local-live-api-conformance-summary.json",
    },
    {
        "scenario_id": "signature_parity",
        "command_snippet": "run_signature_parity_contract_lane.sh",
        "artifact_snippet": "/tmp/kolme-signature-parity-matrix-report.json",
    },
    {
        "scenario_id": "runtime_commit_finality",
        "command_snippet": "run_local_runtime_commit_live_finality_evidence_contract_lane.sh",
        "artifact_snippet": "/tmp/kolme-local-runtime-commit-live-summary.json",
    },
    {
        "scenario_id": "native_api_parity",
        "command_snippet": "run_local_native_api_parity_live_proof_contract_lane.sh",
        "artifact_snippet": "/tmp/kolme-local-native-api-parity-live-proof-summary.json",
    },
    {
        "scenario_id": "real_node_runtime_integration",
        "command_snippet": "run_local_kamn_live_runtime_integration_lane.sh --mode dry-run --runtime-profile real-node",
        "artifact_snippet": "/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
    },
    {
        "scenario_id": "real_node_runtime_policy",
        "command_snippet": "check_local_kamn_live_runtime_real_node_profile_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json",
        "artifact_snippet": "/tmp/kolme-local-kamn-live-runtime-real-node-policy.json",
    },
]

EXPECTED_SCENARIO_IDS = [scenario["scenario_id"] for scenario in EXPECTED_SCENARIO_MATRIX]
EXPECTED_COMMAND_SNIPPETS = [scenario["command_snippet"] for scenario in EXPECTED_SCENARIO_MATRIX]
EXPECTED_ARTIFACT_SNIPPETS = [scenario["artifact_snippet"] for scenario in EXPECTED_SCENARIO_MATRIX] + [
    "/tmp/kolme-local-fork-rust-test-matrix-policy.json",
    "/tmp/kolme-local-live-api-conformance-policy.json",
    "/tmp/kolme-signature-parity-policy-report.json",
    "/tmp/kolme-local-runtime-commit-live-policy.json",
    "/tmp/kolme-local-native-api-parity-live-proof-policy.json",
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

    if report.get("scenario_matrix_schema_version") != SCENARIO_MATRIX_SCHEMA_VERSION:
        reason_codes.append("scenario_matrix_schema_version_mismatch")

    scenario_runtime_mode = report.get("scenario_runtime_mode")
    if scenario_runtime_mode not in ("dry-run", "run"):
        reason_codes.append("scenario_runtime_mode_invalid")

    scenario_runtime_profiles = report.get("scenario_runtime_profiles")
    if not isinstance(scenario_runtime_profiles, list) or not scenario_runtime_profiles:
        reason_codes.append("scenario_runtime_profiles_missing")
    elif scenario_runtime_profiles != ["real-node"]:
        reason_codes.append("scenario_runtime_profiles_mismatch")

    scenario_ids = report.get("scenario_ids")
    if not isinstance(scenario_ids, list) or not scenario_ids:
        reason_codes.append("scenario_ids_missing")
        scenario_ids = []
    else:
        if any(not isinstance(scenario_id, str) or not scenario_id for scenario_id in scenario_ids):
            reason_codes.append("scenario_ids_invalid")
        if sorted(scenario_ids) != sorted(EXPECTED_SCENARIO_IDS):
            reason_codes.append("scenario_ids_mismatch")

    scenario_count = report.get("scenario_count")
    if not isinstance(scenario_count, int) or scenario_count < 0:
        reason_codes.append("scenario_count_invalid")
    elif scenario_count != len(EXPECTED_SCENARIO_IDS):
        reason_codes.append("scenario_count_mismatch")
    if isinstance(scenario_ids, list) and isinstance(scenario_count, int) and scenario_count != len(scenario_ids):
        reason_codes.append("scenario_count_ids_mismatch")

    commands = report.get("commands")
    if not isinstance(commands, list) or not commands:
        reason_codes.append("commands_missing")
        commands = []
    if isinstance(commands, list):
        if not all(isinstance(command, str) and command.strip() for command in commands):
            reason_codes.append("commands_invalid")
        for scenario in EXPECTED_SCENARIO_MATRIX:
            scenario_command = scenario["command_snippet"]
            if not any(
                scenario_command in command
                for command in commands
                if isinstance(command, str)
            ):
                reason_codes.append(f"scenario_command_missing:{scenario['scenario_id']}")
        for expected_snippet in EXPECTED_COMMAND_SNIPPETS:
            if not any(expected_snippet in command for command in commands if isinstance(command, str)):
                reason_codes.append(f"command_missing:{expected_snippet}")
        if not any(
            all(
                marker in command
                for marker in (
                    "run_signature_parity_contract_lane.sh",
                    "KAMN_KOLME_SIGNATURE_PARITY_MAX_SECONDS=120",
                )
            )
            for command in commands
            if isinstance(command, str)
        ):
            reason_codes.append("signature_parity_budget_marker_missing")
        if not any(
            all(
                marker in command
                for marker in (
                    "run_local_runtime_commit_live_finality_evidence_contract_lane.sh",
                    "--max-seconds 120",
                    "--finality-max-seconds 15",
                    "--require-non-synthetic-run-evidence",
                    "--require-native-payload-evidence",
                )
            )
            for command in commands
            if isinstance(command, str)
        ):
            reason_codes.append("native_runtime_commit_budget_marker_missing")
        if not any(
            all(
                marker in command
                for marker in (
                    "run_local_native_api_parity_live_proof_contract_lane.sh",
                    "--max-seconds 180",
                )
            )
            for command in commands
            if isinstance(command, str)
        ):
            reason_codes.append("native_api_parity_budget_marker_missing")
        if not any(
            all(
                marker in command
                for marker in (
                    "run_local_kamn_live_runtime_integration_lane.sh",
                    "--mode dry-run",
                    "--runtime-profile real-node",
                    "--max-seconds 210",
                    "--runtime-commit-max-seconds 30",
                    "--runtime-commit-finality-max-seconds 15",
                )
            )
            for command in commands
            if isinstance(command, str)
        ):
            reason_codes.append("native_real_node_budget_marker_missing")
        if not any(
            all(
                marker in command
                for marker in (
                    "check_local_kamn_live_runtime_real_node_profile_policy.py",
                    "--require-non-synthetic-run-evidence",
                )
            )
            for command in commands
            if isinstance(command, str)
        ):
            reason_codes.append("native_real_node_policy_marker_missing")

    artifact_paths = report.get("artifact_paths")
    if not isinstance(artifact_paths, list) or not artifact_paths:
        reason_codes.append("artifact_paths_missing")
        artifact_paths = []
    if isinstance(artifact_paths, list):
        if not all(isinstance(path, str) and path.strip() for path in artifact_paths):
            reason_codes.append("artifact_paths_invalid")
        for scenario in EXPECTED_SCENARIO_MATRIX:
            scenario_artifact = scenario["artifact_snippet"]
            if not any(
                scenario_artifact in path
                for path in artifact_paths
                if isinstance(path, str)
            ):
                reason_codes.append(f"scenario_artifact_missing:{scenario['scenario_id']}")
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
    if (
        isinstance(mode, str)
        and isinstance(scenario_runtime_mode, str)
        and mode in ("dry-run", "run")
        and scenario_runtime_mode in ("dry-run", "run")
        and scenario_runtime_mode != mode
    ):
        reason_codes.append("scenario_runtime_mode_mismatch")

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
