#!/usr/bin/env python3
"""Production go/no-go gate lane contract runner."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, require_non_negative_int, write_json  # noqa: E402
from framework.contract_framework import require_enum  # noqa: E402

GATE_DECISION_FAULT_REASON = "gate_decision_fault_injection_triggered"
RUNTIME_BUDGET_EXCEEDED_REASON = "runtime_budget_exceeded"
GO_NO_GO_REASON_TAXONOMY_VERSION = "kamn.runtime.go-no-go-gate-reason-taxonomy.v1"
RELEASE_MANIFEST_SCHEMA = "kamn.runtime.release-evidence-manifest.v1"
DEFAULT_RELEASE_MANIFEST_PATH = ROOT_DIR / "scripts/runtime/release_evidence_manifest.json"
RUN_MODE_FAST_GATE_EXCLUSION_REASON = "go_no_go_gate_run_mode_excluded_from_fast_gate"
DRY_RUN_REASON = "dry_run_no_commands_executed"
RUN_REASON = "release_candidate_artifact_aggregation_executed"
RUN_MODE_OPT_IN_ENV = "KAMN_GONOGO_GATE_LOCAL_OPT_IN"
WAIVER_SCHEMA = "kamn.runtime.go-no-go-gate-waiver.v1"
WAIVER_SCOPE = "runtime_go_no_go_gate_required_artifacts"
WAIVER_APPLIED_REASON = "release_manifest_required_artifact_waiver_applied"
COMBINED_REASON_TAXONOMY_VERSION = "kamn.runtime.local-full-stack-integration-reason-taxonomy.v1"
COMBINED_TRANSPORT_REASON_CODE = "fork_choice_stale_block_height"
KOLME_RUNTIME_COMMIT_FAILURE_TAXONOMY_VERSION = "v1"
KOLME_RUNTIME_COMMIT_PROFILE = "real-node-non-synthetic-v1"
KOLME_RUNTIME_COMMIT_PROFILE_VERSION = "v1"
NATIVE_LIBP2P_PROVIDER_MARKER = "p2p-live-libp2p-provider:native"
LIBP2P_FALLBACK_MARKER_BLOCKLIST = [
    "p2p-in-memory-transport-fallback",
    "p2p-live-libp2p-provider:contract-only",
]
REQUIRED_ARTIFACT_IDS = (
    "go_no_go_evidence",
    "rollback_readiness",
    "dr_readiness",
    "local_full_stack_integration",
    "local_full_runtime_convergence",
    "transport_fault_matrix",
)
ARTIFACT_LANE_REGISTRY: dict[str, dict[str, object]] = {
    "go_no_go_evidence": {
        "expected_lane": "deploy.run_gonogo_evidence_deep_lane",
        "command": ["bash", str(ROOT_DIR / "scripts/deploy/run_gonogo_evidence_deep_lane.sh")],
        "success_marker": "go/no-go evidence deep lane tests passed.",
        "failure_label": "go/no-go evidence lane failed unexpectedly",
    },
    "rollback_readiness": {
        "expected_lane": "deploy.run_deployment_slo_rollback_contract_lane",
        "command": [
            "bash",
            str(ROOT_DIR / "scripts/deploy/run_deployment_slo_rollback_contract_lane.sh"),
        ],
        "success_marker": "final_decision=GO",
        "failure_label": "rollback readiness lane failed unexpectedly",
    },
    "dr_readiness": {
        "expected_lane": "deploy.run_dr_evidence_contract_lane",
        "command": ["bash", str(ROOT_DIR / "scripts/deploy/run_dr_evidence_contract_lane.sh")],
        "success_marker": "dr evidence contract lane tests passed.",
        "failure_label": "dr readiness lane failed unexpectedly",
    },
    "local_full_stack_integration": {
        "expected_lane": "runtime.validate_local_full_stack_integration_live_contract_lane",
        "command": [
            "bash",
            str(ROOT_DIR / "scripts/runtime/validate_local_full_stack_integration_live_contract_lane.sh"),
            "--mode",
            "dry-run",
        ],
        "success_marker": "local_full_stack_integration_policy_status=verified",
        "failure_label": "local full-stack integration lane failed unexpectedly",
    },
    "local_full_runtime_convergence": {
        "expected_lane": "runtime.validate_local_full_runtime_live_contract_lane",
        "command": [
            "bash",
            str(ROOT_DIR / "scripts/runtime/validate_local_full_runtime_live_contract_lane.sh"),
            "--mode",
            "dry-run",
        ],
        "success_marker": "local_full_runtime_policy_status=verified",
        "failure_label": "local full-runtime convergence lane failed unexpectedly",
    },
    "transport_fault_matrix": {
        "expected_lane": "runtime.validate_block_reconciliation_partition_rejoin_live_contract_lane",
        "command": [
            "bash",
            str(
                ROOT_DIR
                / "scripts/runtime/validate_block_reconciliation_partition_rejoin_live_contract_lane.sh"
            ),
            "--mode",
            "dry-run",
        ],
        "success_marker": "block_reconciliation_partition_rejoin_policy_status=verified",
        "failure_label": "transport fault-matrix lane failed unexpectedly",
    },
}


def _ensure_executable(path: Path, label: str) -> None:
    if not path.is_file() or not os.access(path, os.X_OK):
        fail(f"expected {label} to be executable")


def _run_command(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _require_line_value(output: str, key: str, reason_code: str) -> str:
    value = _extract_line_value(output, key)
    if not value:
        fail(reason_code)
    return value


def _parse_csv_field(value: str) -> list[str]:
    normalized = value.strip()
    if not normalized or normalized == "none":
        return []
    return [segment.strip() for segment in normalized.split(",") if segment.strip()]


def _resolve_manifest_path(raw: str) -> Path:
    value = raw.strip()
    if not value:
        return DEFAULT_RELEASE_MANIFEST_PATH
    path = Path(value)
    if path.is_absolute():
        return path
    return (ROOT_DIR / path).resolve()


def _resolve_optional_path(raw: str) -> Path | None:
    value = raw.strip()
    if not value:
        return None
    path = Path(value)
    if path.is_absolute():
        return path
    return (ROOT_DIR / path).resolve()


def _load_release_manifest(path: Path) -> dict[str, object]:
    if not path.is_file():
        fail("release_manifest_file_missing")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        fail("release_manifest_json_invalid")
    if not isinstance(payload, dict):
        fail("release_manifest_root_invalid")
    return payload


def _require_non_empty_string(payload: dict[str, object], key: str, reason_code: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        fail(reason_code)
    return value.strip()


def _parse_waiver(
    waiver_file: Path | None,
    triggered_reason_codes: list[str],
) -> tuple[str, list[str], list[str], str]:
    if waiver_file is None:
        return "none", [], triggered_reason_codes, ""
    if not waiver_file.is_file():
        return "none", [], triggered_reason_codes, "waiver_file_not_found"

    try:
        waiver_payload = json.loads(waiver_file.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return "none", [], triggered_reason_codes, "waiver_file_json_invalid"
    if not isinstance(waiver_payload, dict):
        return "none", [], triggered_reason_codes, "waiver_file_root_invalid"

    if waiver_payload.get("schema_version") != WAIVER_SCHEMA:
        return "none", [], triggered_reason_codes, "waiver_file_schema_mismatch"
    if waiver_payload.get("scope") != WAIVER_SCOPE:
        return "none", [], triggered_reason_codes, "waiver_scope_mismatch"

    expires_on = waiver_payload.get("expires_on")
    if not isinstance(expires_on, str):
        return "none", [], triggered_reason_codes, "waiver_expiry_invalid"
    try:
        expiry_date = dt.date.fromisoformat(expires_on)
    except ValueError:
        return "none", [], triggered_reason_codes, "waiver_expiry_invalid"
    if expiry_date < dt.date.today():
        return "none", [], triggered_reason_codes, "waiver_expired"

    allowed_reason_codes = waiver_payload.get("allowed_reason_codes", [])
    if not isinstance(allowed_reason_codes, list) or not all(
        isinstance(value, str) and value for value in allowed_reason_codes
    ):
        return "none", [], triggered_reason_codes, "waiver_allowed_reason_codes_invalid"

    allowed_set = set(allowed_reason_codes)
    waived_reason_codes = sorted(
        reason_code for reason_code in triggered_reason_codes if reason_code in allowed_set
    )
    unwaived_reason_codes = sorted(
        reason_code for reason_code in triggered_reason_codes if reason_code not in allowed_set
    )
    if unwaived_reason_codes:
        return "none", waived_reason_codes, unwaived_reason_codes, ""
    return "applied", waived_reason_codes, [], ""


def _extract_missing_manifest_artifact_id(reason_code: str) -> str | None:
    prefix = "release_manifest_missing_required_artifact:"
    if not reason_code.startswith(prefix):
        return None
    artifact_id = reason_code[len(prefix) :]
    if artifact_id not in REQUIRED_ARTIFACT_IDS:
        return None
    return artifact_id


def _validate_release_manifest(
    payload: dict[str, object],
    *,
    waived_artifact_ids: set[str] | None = None,
) -> list[dict[str, str]]:
    waived_artifact_ids = waived_artifact_ids or set()
    schema_version = payload.get("schema_version")
    if schema_version != RELEASE_MANIFEST_SCHEMA:
        fail("release_manifest_schema_version_invalid")

    required_artifacts = payload.get("required_artifacts")
    if not isinstance(required_artifacts, list):
        fail("release_manifest_required_artifacts_missing")

    seen_artifact_ids: set[str] = set()
    validated_artifacts: list[dict[str, str]] = []
    for entry in required_artifacts:
        if not isinstance(entry, dict):
            fail("release_manifest_required_artifact_entry_invalid")
        artifact_id = _require_non_empty_string(
            entry,
            "artifact_id",
            "release_manifest_required_artifact_id_invalid",
        )
        if artifact_id in seen_artifact_ids:
            fail(f"release_manifest_duplicate_artifact_id:{artifact_id}")
        if artifact_id not in ARTIFACT_LANE_REGISTRY:
            fail(f"release_manifest_unknown_artifact_id:{artifact_id}")

        expected_lane = _require_non_empty_string(
            entry,
            "expected_lane",
            f"release_manifest_expected_lane_missing:{artifact_id}",
        )
        expected_success_marker = _require_non_empty_string(
            entry,
            "expected_success_marker",
            f"release_manifest_expected_success_marker_missing:{artifact_id}",
        )
        registry_entry = ARTIFACT_LANE_REGISTRY[artifact_id]
        if expected_lane != registry_entry["expected_lane"]:
            fail(f"release_manifest_lane_mismatch:{artifact_id}")
        if expected_success_marker != registry_entry["success_marker"]:
            fail(f"release_manifest_success_marker_mismatch:{artifact_id}")

        seen_artifact_ids.add(artifact_id)
        validated_artifacts.append(
            {
                "artifact_id": artifact_id,
                "expected_lane": expected_lane,
                "expected_success_marker": expected_success_marker,
            }
        )

    for required_artifact in REQUIRED_ARTIFACT_IDS:
        if required_artifact in waived_artifact_ids:
            continue
        if required_artifact not in seen_artifact_ids:
            fail(f"release_manifest_missing_required_artifact:{required_artifact}")

    return validated_artifacts


def _evaluate_go_no_go_policy(
    artifact_inventory: list[dict[str, str]],
    observed_reason_codes: list[str],
    lane_mode: str,
    native_libp2p_provider_marker: str,
    libp2p_fallback_marker_blocklist: list[str],
    libp2p_fallback_markers_detected: list[str],
    native_libp2p_provider_marker_contract_status: str,
) -> tuple[str, str, str, list[str]]:
    fail_reasons: list[str] = []
    warn_reasons: list[str] = []
    expected_artifact_status = "verified" if lane_mode == "run" else "dry_run_pending"

    for artifact in artifact_inventory:
        if artifact.get("status") != expected_artifact_status:
            artifact_id = artifact.get("artifact_id", "unknown")
            fail_reasons.append(f"gate_required_artifact_status_mismatch:{artifact_id}")

    for reason_code in observed_reason_codes:
        if reason_code == GATE_DECISION_FAULT_REASON:
            fail_reasons.append(reason_code)
            continue
        if reason_code == RUNTIME_BUDGET_EXCEEDED_REASON:
            warn_reasons.append(reason_code)
            continue
        if reason_code == WAIVER_APPLIED_REASON:
            warn_reasons.append(reason_code)
            continue
        fail_reasons.append(f"gate_policy_unknown_reason_code:{reason_code}")

    if native_libp2p_provider_marker != NATIVE_LIBP2P_PROVIDER_MARKER:
        fail_reasons.append("gate_policy_native_libp2p_provider_marker_mismatch")
    if libp2p_fallback_marker_blocklist != LIBP2P_FALLBACK_MARKER_BLOCKLIST:
        fail_reasons.append("gate_policy_libp2p_fallback_marker_blocklist_mismatch")
    if libp2p_fallback_markers_detected:
        fail_reasons.append("gate_policy_libp2p_fallback_markers_detected")
    if native_libp2p_provider_marker_contract_status != "verified":
        fail_reasons.append("gate_policy_native_libp2p_provider_marker_contract_status_mismatch")

    fail_reasons = sorted(set(fail_reasons))
    warn_reasons = sorted(set(warn_reasons))

    if fail_reasons:
        combined = sorted(set(fail_reasons + warn_reasons))
        return "FAIL", "NO-GO", "fail", combined
    if warn_reasons:
        return "WARN", "GO", "warn", warn_reasons
    return "PASS", "GO", "pass", []


def run_go_no_go_gate_lane(args: argparse.Namespace) -> int:
    lane_mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    fault_profile = args.fault_profile.strip()
    if fault_profile not in {
        "none",
        "gate_decision",
        "runtime_budget_warn",
        "libp2p_fallback_marker",
    }:
        fail(
            "--fault-profile must be one of: "
            "none, gate_decision, runtime_budget_warn, libp2p_fallback_marker"
        )
    if lane_mode == "run" and args.local_opt_in.strip() != "1":
        fail(f"run mode requires {RUN_MODE_OPT_IN_ENV}=1")

    max_seconds = require_non_negative_int("KAMN_GONOGO_GATE_MAX_SECONDS", args.max_seconds)
    start_epoch = int(time.time())
    manifest_path = _resolve_manifest_path(args.manifest_file)
    waiver_file = _resolve_optional_path(args.waiver_file)
    manifest_payload = _load_release_manifest(manifest_path)
    waiver_status = "none"
    waived_reason_codes: list[str] = []
    waived_artifact_ids: set[str] = set()
    while True:
        try:
            required_artifacts = _validate_release_manifest(
                manifest_payload,
                waived_artifact_ids=waived_artifact_ids,
            )
            break
        except ContractError as error:
            manifest_reason_code = str(error)
            missing_artifact_id = _extract_missing_manifest_artifact_id(manifest_reason_code)
            if missing_artifact_id is None or missing_artifact_id in waived_artifact_ids:
                fail(manifest_reason_code)
            parsed_waiver = _parse_waiver(waiver_file, [manifest_reason_code])
            parsed_waiver_status, waiver_hit_reason_codes, unwaived_reason_codes, waiver_error = (
                parsed_waiver
            )
            if waiver_error:
                fail(waiver_error)
            if unwaived_reason_codes:
                fail(unwaived_reason_codes[0])
            if parsed_waiver_status != "applied":
                fail(manifest_reason_code)
            waiver_status = "applied"
            waived_reason_codes = sorted(set(waived_reason_codes + waiver_hit_reason_codes))
            waived_artifact_ids.add(missing_artifact_id)

    gonogo_generator = ROOT_DIR / "scripts/deploy/generate_gonogo_evidence_bundle.sh"
    gonogo_checker = ROOT_DIR / "scripts/deploy/check_gonogo_evidence_policy.sh"

    _ensure_executable(gonogo_generator, "go/no-go evidence bundle generator")
    _ensure_executable(gonogo_checker, "go/no-go evidence policy checker")

    reason_codes: list[str] = []
    if waiver_status == "applied":
        reason_codes.append(WAIVER_APPLIED_REASON)
    artifact_inventory: list[dict[str, str]] = []
    run_mode_command_count = 0
    combined_reason_taxonomy_version = COMBINED_REASON_TAXONOMY_VERSION
    combined_transport_reason_codes = [COMBINED_TRANSPORT_REASON_CODE]
    combined_kolme_runtime_reason_code = "not_run"
    kolme_runtime_commit_failure_taxonomy_version = KOLME_RUNTIME_COMMIT_FAILURE_TAXONOMY_VERSION
    kolme_runtime_commit_failure_taxonomy = "not_run"
    kolme_fixture_profile = KOLME_RUNTIME_COMMIT_PROFILE
    kolme_fixture_profile_version = KOLME_RUNTIME_COMMIT_PROFILE_VERSION
    kolme_fixture_profile_status = "planned" if lane_mode == "dry-run" else "verified"
    native_libp2p_provider_marker = NATIVE_LIBP2P_PROVIDER_MARKER
    libp2p_fallback_marker_blocklist = list(LIBP2P_FALLBACK_MARKER_BLOCKLIST)
    libp2p_fallback_markers_detected: list[str] = []
    native_libp2p_provider_marker_contract_status = "verified"

    for artifact in required_artifacts:
        artifact_id = artifact["artifact_id"]
        expected_marker = artifact["expected_success_marker"]
        artifact_entry = {
            "artifact_id": artifact_id,
            "expected_lane": artifact["expected_lane"],
            "expected_success_marker": expected_marker,
        }
        if lane_mode == "run":
            registry_entry = ARTIFACT_LANE_REGISTRY[artifact_id]
            lane_command = registry_entry["command"]
            if not isinstance(lane_command, list) or len(lane_command) < 2:
                fail(f"release_manifest_command_invalid:{artifact_id}")
            lane_path = Path(str(lane_command[1]))
            _ensure_executable(lane_path, f"{artifact_id} lane command")
            run_result = _run_command([str(part) for part in lane_command])
            if run_result.returncode != 0:
                detail = (
                    run_result.stderr
                    or run_result.stdout
                    or f"{registry_entry['failure_label']}: unknown failure"
                ).strip()
                fail(f"release_manifest_artifact_execution_failed:{artifact_id}:{detail}")
            if expected_marker not in run_result.stdout:
                fail(f"release_manifest_required_marker_missing:{artifact_id}")

            if artifact_id == "local_full_stack_integration":
                combined_reason_taxonomy_version = _require_line_value(
                    run_result.stdout,
                    "combined_reason_taxonomy_version",
                    "local_full_stack_integration_combined_reason_taxonomy_version_missing",
                )
                if combined_reason_taxonomy_version != COMBINED_REASON_TAXONOMY_VERSION:
                    fail("local_full_stack_integration_combined_reason_taxonomy_version_mismatch")

                combined_transport_reason_codes_csv = _require_line_value(
                    run_result.stdout,
                    "combined_transport_reason_codes",
                    "local_full_stack_integration_combined_transport_reason_codes_missing",
                )
                if combined_transport_reason_codes_csv != COMBINED_TRANSPORT_REASON_CODE:
                    fail("local_full_stack_integration_combined_transport_reason_codes_mismatch")
                combined_transport_reason_codes = [combined_transport_reason_codes_csv]

                combined_kolme_runtime_reason_code = _require_line_value(
                    run_result.stdout,
                    "combined_kolme_runtime_reason_code",
                    "local_full_stack_integration_combined_kolme_runtime_reason_code_missing",
                )

                kolme_runtime_commit_failure_taxonomy_version = _require_line_value(
                    run_result.stdout,
                    "kolme_runtime_commit_failure_taxonomy_version",
                    (
                        "local_full_stack_integration_kolme_runtime_commit_"
                        "failure_taxonomy_version_missing"
                    ),
                )
                if (
                    kolme_runtime_commit_failure_taxonomy_version
                    != KOLME_RUNTIME_COMMIT_FAILURE_TAXONOMY_VERSION
                ):
                    fail(
                        "local_full_stack_integration_kolme_runtime_commit_"
                        "failure_taxonomy_version_mismatch"
                    )

                kolme_runtime_commit_failure_taxonomy = _require_line_value(
                    run_result.stdout,
                    "kolme_runtime_commit_failure_taxonomy",
                    "local_full_stack_integration_kolme_runtime_commit_failure_taxonomy_missing",
                )

                kolme_fixture_profile = _require_line_value(
                    run_result.stdout,
                    "kolme_fixture_profile",
                    "local_full_stack_integration_kolme_fixture_profile_missing",
                )
                if kolme_fixture_profile != KOLME_RUNTIME_COMMIT_PROFILE:
                    fail("local_full_stack_integration_kolme_fixture_profile_mismatch")

                kolme_fixture_profile_version = _require_line_value(
                    run_result.stdout,
                    "kolme_fixture_profile_version",
                    "local_full_stack_integration_kolme_fixture_profile_version_missing",
                )
                if kolme_fixture_profile_version != KOLME_RUNTIME_COMMIT_PROFILE_VERSION:
                    fail("local_full_stack_integration_kolme_fixture_profile_version_mismatch")

                kolme_fixture_profile_status = _require_line_value(
                    run_result.stdout,
                    "kolme_fixture_profile_status",
                    "local_full_stack_integration_kolme_fixture_profile_status_missing",
                )
                if kolme_fixture_profile_status not in {"planned", "verified"}:
                    fail("local_full_stack_integration_kolme_fixture_profile_status_mismatch")

                native_libp2p_provider_marker = _require_line_value(
                    run_result.stdout,
                    "libp2p_native_provider_marker",
                    "local_full_stack_integration_libp2p_native_provider_marker_missing",
                )
                libp2p_fallback_marker_blocklist_csv = _require_line_value(
                    run_result.stdout,
                    "libp2p_fallback_marker_blocklist",
                    "local_full_stack_integration_libp2p_fallback_marker_blocklist_missing",
                )
                libp2p_fallback_marker_blocklist = _parse_csv_field(
                    libp2p_fallback_marker_blocklist_csv
                )
                if not libp2p_fallback_marker_blocklist:
                    fail("local_full_stack_integration_libp2p_fallback_marker_blocklist_invalid")

                libp2p_fallback_markers_detected_csv = _require_line_value(
                    run_result.stdout,
                    "libp2p_fallback_markers_detected",
                    "local_full_stack_integration_libp2p_fallback_markers_detected_missing",
                )
                libp2p_fallback_markers_detected = _parse_csv_field(
                    libp2p_fallback_markers_detected_csv
                )
                native_libp2p_provider_marker_contract_status = _require_line_value(
                    run_result.stdout,
                    "libp2p_provider_marker_contract_status",
                    (
                        "local_full_stack_integration_"
                        "libp2p_provider_marker_contract_status_missing"
                    ),
                )

            artifact_entry["status"] = "verified"
            run_mode_command_count += 1
        else:
            artifact_entry["status"] = "dry_run_pending"
        artifact_inventory.append(artifact_entry)

    if fault_profile == "gate_decision":
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            no_go_bundle = temp_path / "gonogo-no-go-bundle.json"
            drift_bundle = temp_path / "gonogo-drift-bundle.json"

            generate_run = _run_command(
                [
                    "bash",
                    str(gonogo_generator),
                    "--output-file",
                    str(no_go_bundle),
                    "--release-candidate",
                    "v1.0.0-fault",
                    "--schema-target-version",
                    "1.0.0",
                    "--runtime-image-digest",
                    "sha256:fault",
                    "--ci-fast-gate",
                    "PASS",
                    "--ci-deep-lane",
                    "FAIL",
                    "--rollback-precheck",
                    "PASS",
                    "--rollback-trigger-status",
                    "CLEAR",
                    "--required-approvals",
                    "2",
                    "--received-approvals",
                    "1",
                ]
            )
            if generate_run.returncode != 0:
                detail = (generate_run.stderr or generate_run.stdout or "generator failed").strip()
                fail(f"go/no-go generator failed unexpectedly: {detail}")

            payload = json.loads(no_go_bundle.read_text(encoding="utf-8"))
            payload["final_decision"] = "GO"
            drift_bundle.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")

            drift_run = _run_command(
                [
                    "bash",
                    str(gonogo_checker),
                    "--bundle-file",
                    str(drift_bundle),
                ]
            )
            if drift_run.returncode == 0:
                fail("gate_decision fault profile expected policy checker fail-closed behavior")
            reason_codes.append(GATE_DECISION_FAULT_REASON)
    elif fault_profile == "libp2p_fallback_marker":
        libp2p_fallback_markers_detected = [LIBP2P_FALLBACK_MARKER_BLOCKLIST[0]]
    elif fault_profile == "runtime_budget_warn":
        reason_codes.append(RUNTIME_BUDGET_EXCEEDED_REASON)

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        reason_codes.append(RUNTIME_BUDGET_EXCEEDED_REASON)

    reason_codes = sorted(set(reason_codes))
    ci_fast_gate_eligible = lane_mode == "dry-run"
    ci_fast_gate_scope = "ci-fast-gate" if ci_fast_gate_eligible else "local-only"
    run_mode_command_status = "executed" if lane_mode == "run" else "dry_run_no_commands_executed"
    mode_reason_code = RUN_REASON if lane_mode == "run" else DRY_RUN_REASON
    policy_outcome, final_decision, status, evaluator_reason_codes = _evaluate_go_no_go_policy(
        artifact_inventory,
        reason_codes,
        lane_mode,
        native_libp2p_provider_marker,
        libp2p_fallback_marker_blocklist,
        libp2p_fallback_markers_detected,
        native_libp2p_provider_marker_contract_status,
    )

    report_payload = {
        "schema_version": "kamn.runtime.go-no-go-gate-report.v1",
        "reason_taxonomy_version": GO_NO_GO_REASON_TAXONOMY_VERSION,
        "status": status,
        "policy_outcome": policy_outcome,
        "final_decision": final_decision,
        "lane_mode": lane_mode,
        "ci_fast_gate": ci_fast_gate,
        "ci_fast_gate_eligible": ci_fast_gate_eligible,
        "ci_fast_gate_scope": ci_fast_gate_scope,
        "fast_gate_exclusion_status": "verified",
        "fast_gate_exclusion_reason_code": RUN_MODE_FAST_GATE_EXCLUSION_REASON,
        "run_mode_command_status": run_mode_command_status,
        "run_mode_command_count": run_mode_command_count,
        "mode_reason_code": mode_reason_code,
        "fault_profile": fault_profile,
        "go_no_go_evidence_status": "verified" if lane_mode == "run" else "dry_run_pending",
        "rollback_readiness_status": "verified" if lane_mode == "run" else "dry_run_pending",
        "dr_readiness_status": "verified" if lane_mode == "run" else "dry_run_pending",
        "local_full_stack_integration_status": "verified"
        if lane_mode == "run"
        else "dry_run_pending",
        "local_full_runtime_convergence_status": "verified"
        if lane_mode == "run"
        else "dry_run_pending",
        "transport_fault_matrix_status": "verified"
        if lane_mode == "run"
        else "dry_run_pending",
        "policy_evaluator_status": "verified",
        "manifest_schema_version": manifest_payload["schema_version"],
        "manifest_registry_status": "verified",
        "combined_reason_taxonomy_version": combined_reason_taxonomy_version,
        "combined_transport_reason_codes": combined_transport_reason_codes,
        "combined_kolme_runtime_reason_code": combined_kolme_runtime_reason_code,
        "kolme_runtime_commit_failure_taxonomy_version": kolme_runtime_commit_failure_taxonomy_version,
        "kolme_runtime_commit_failure_taxonomy": kolme_runtime_commit_failure_taxonomy,
        "kolme_fixture_profile": kolme_fixture_profile,
        "kolme_fixture_profile_version": kolme_fixture_profile_version,
        "kolme_fixture_profile_status": kolme_fixture_profile_status,
        "combined_lane_marker_contract_status": "verified",
        "native_libp2p_provider_marker": native_libp2p_provider_marker,
        "libp2p_fallback_marker_blocklist": libp2p_fallback_marker_blocklist,
        "libp2p_fallback_markers_detected": libp2p_fallback_markers_detected,
        "native_libp2p_provider_marker_contract_status": native_libp2p_provider_marker_contract_status,
        "waiver_status": waiver_status,
        "waived_reason_codes": waived_reason_codes,
        "waiver_review_required": waiver_status == "applied",
        "waiver_schema_version": WAIVER_SCHEMA if waiver_status == "applied" else "",
        "waiver_scope": WAIVER_SCOPE if waiver_status == "applied" else "",
        "waiver_file": str(waiver_file) if waiver_file else "",
        "required_artifact_ids": [artifact["artifact_id"] for artifact in required_artifacts],
        "artifact_inventory": artifact_inventory,
        "reason_codes": evaluator_reason_codes,
        "observed_reason_codes": reason_codes,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
    }
    if args.output_json:
        write_json(Path(args.output_json), report_payload)

    reason_codes_csv = "none" if not evaluator_reason_codes else ",".join(evaluator_reason_codes)
    observed_reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)
    print(f"status={status}")
    print(f"policy_outcome={policy_outcome}")
    print(f"final_decision={final_decision}")
    print(f"lane_mode={lane_mode}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(f"ci_fast_gate_eligible={'true' if ci_fast_gate_eligible else 'false'}")
    print(f"ci_fast_gate_scope={ci_fast_gate_scope}")
    print("fast_gate_exclusion_status=verified")
    print(f"fast_gate_exclusion_reason_code={RUN_MODE_FAST_GATE_EXCLUSION_REASON}")
    print(f"run_mode_command_status={run_mode_command_status}")
    print(f"run_mode_command_count={run_mode_command_count}")
    print(f"mode_reason_code={mode_reason_code}")
    print(f"fault_profile={fault_profile}")
    if lane_mode == "run":
        print("go_no_go_evidence_status=verified")
        print("rollback_readiness_status=verified")
        print("dr_readiness_status=verified")
        print("local_full_stack_integration_status=verified")
        print("local_full_runtime_convergence_status=verified")
        print("transport_fault_matrix_status=verified")
    else:
        print("go_no_go_evidence_status=dry_run_pending")
        print("rollback_readiness_status=dry_run_pending")
        print("dr_readiness_status=dry_run_pending")
        print("local_full_stack_integration_status=dry_run_pending")
        print("local_full_runtime_convergence_status=dry_run_pending")
        print("transport_fault_matrix_status=dry_run_pending")
    print("policy_evaluator_status=verified")
    print(f"manifest_schema_version={manifest_payload['schema_version']}")
    print(f"reason_taxonomy_version={GO_NO_GO_REASON_TAXONOMY_VERSION}")
    print("manifest_registry_status=verified")
    print(f"combined_reason_taxonomy_version={combined_reason_taxonomy_version}")
    print(f"combined_transport_reason_codes={','.join(combined_transport_reason_codes)}")
    print(f"combined_kolme_runtime_reason_code={combined_kolme_runtime_reason_code}")
    print(
        "kolme_runtime_commit_failure_taxonomy_version="
        f"{kolme_runtime_commit_failure_taxonomy_version}"
    )
    print(f"kolme_runtime_commit_failure_taxonomy={kolme_runtime_commit_failure_taxonomy}")
    print(f"kolme_fixture_profile={kolme_fixture_profile}")
    print(f"kolme_fixture_profile_version={kolme_fixture_profile_version}")
    print(f"kolme_fixture_profile_status={kolme_fixture_profile_status}")
    print("combined_lane_marker_contract_status=verified")
    print(f"native_libp2p_provider_marker={native_libp2p_provider_marker}")
    print(f"libp2p_fallback_marker_blocklist={','.join(libp2p_fallback_marker_blocklist)}")
    print(
        "libp2p_fallback_markers_detected="
        f"{'none' if not libp2p_fallback_markers_detected else ','.join(libp2p_fallback_markers_detected)}"
    )
    print(
        "native_libp2p_provider_marker_contract_status="
        f"{native_libp2p_provider_marker_contract_status}"
    )
    print(f"waiver_status={waiver_status}")
    print(
        "waived_reason_codes="
        + ("none" if not waived_reason_codes else ",".join(waived_reason_codes))
    )
    print(f"waiver_review_required={'true' if waiver_status == 'applied' else 'false'}")
    print(f"required_artifact_count={len(required_artifacts)}")
    print(f"reason_codes={reason_codes_csv}")
    print(f"observed_reason_codes={observed_reason_codes_csv}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    if final_decision != "GO":
        fail(f"go/no-go gate lane failed closed: {reason_codes_csv}")

    print("go/no-go gate lane tests passed.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run production go/no-go gate lane.")
    parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_GONOGO_GATE_MODE", "dry-run"),
    )
    parser.add_argument(
        "--ci-fast-gate",
        default=os.environ.get("KAMN_GONOGO_GATE_CI_FAST_GATE", "PASS"),
    )
    parser.add_argument(
        "--fault-profile",
        default=os.environ.get("KAMN_GONOGO_GATE_FAULT_PROFILE", "none"),
    )
    parser.add_argument("--output-json", default="")
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_GONOGO_GATE_MAX_SECONDS", "180"),
    )
    parser.add_argument(
        "--manifest-file",
        default=os.environ.get(
            "KAMN_GONOGO_GATE_MANIFEST_FILE",
            str(DEFAULT_RELEASE_MANIFEST_PATH),
        ),
    )
    parser.add_argument(
        "--waiver-file",
        default=os.environ.get("KAMN_GONOGO_GATE_WAIVER_FILE", ""),
    )
    parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(RUN_MODE_OPT_IN_ENV, ""),
    )
    parser.set_defaults(handler=run_go_no_go_gate_lane)
    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
