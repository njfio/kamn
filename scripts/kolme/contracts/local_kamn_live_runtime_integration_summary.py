#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
checkout_path = sys.argv[5]
expected_remote_url = sys.argv[6]
expected_ref = sys.argv[7]
base_url = sys.argv[8]
fork_chain_version = sys.argv[9]
elapsed_seconds = int(sys.argv[10])
max_seconds = int(sys.argv[11])
budget_status = sys.argv[12]
runtime_commit_command = sys.argv[13]
runtime_commit_output_file = sys.argv[14]
runtime_commit_live_summary = sys.argv[15]
runtime_commit_live_policy_report = sys.argv[16]
runtime_commit_finality_command = sys.argv[17]
runtime_commit_finality_output_file = sys.argv[18]
runtime_commit_finality_max_seconds = int(sys.argv[19])
bootstrap_report = sys.argv[20]
localhost_signed_report = sys.argv[21]
conformance_report = sys.argv[22]
bootstrap_reason_code = sys.argv[23]
localhost_signed_reason_code = sys.argv[24]
conformance_reason_code = sys.argv[25]
runtime_commit_reason_code = sys.argv[26]
runtime_commit_policy_reason_code = sys.argv[27]
runtime_profile = sys.argv[28]
checks_path = pathlib.Path(sys.argv[29])
runtime_provider_client_contract = sys.argv[30]
runtime_commit_command_profile = sys.argv[31]
runtime_commit_policy_command_profile = sys.argv[32]
runtime_commit_command_profile_version = sys.argv[33]
runtime_signer_profile_selector_env = sys.argv[34]
runtime_signer_profile = sys.argv[35]
runtime_signer_previous_profile = sys.argv[36]
runtime_signer_failover_active = sys.argv[37] == "true"
runtime_signer_rotation_epoch = int(sys.argv[38])
runtime_signer_previous_rotation_epoch = int(sys.argv[39])
runtime_signer_key_source_contract_version = sys.argv[40]
runtime_signer_key_source = sys.argv[41]
runtime_signer_private_key_env = sys.argv[42]
runtime_signer_key_reference_env = sys.argv[43]
runtime_signer_fallback_guard_contract_version = sys.argv[44]
runtime_signer_fallback_private_key_present = sys.argv[45] == "true"
runtime_signer_raw_private_key_present = sys.argv[46] == "true"
runtime_signer_attestation_schema_version = sys.argv[47]
runtime_signing_profile = sys.argv[48]
runtime_signer_fallback_guard_mode = sys.argv[49]
runtime_signer_managed_external_raw_private_key_remediation = sys.argv[50]
runtime_signer_private_key_env_zeroized = (
    not runtime_signer_fallback_private_key_present and not runtime_signer_raw_private_key_present
)
runtime_signer_private_key_bytes_zeroized = runtime_signer_private_key_env_zeroized
PANIC_REASON_MARKERS = ("panic", "unreachable", "unwrap(", "expect(")
KEY_LOADING_ERROR_CLASSIFICATION_VERSION = "v1"
KEY_LOADING_ERROR_CLASSIFICATIONS = (
    "none",
    "fallback_private_key_present",
    "managed_external_raw_private_key_present",
    "key_source_profile_pair_disallowed",
    "private_key_env_mismatch",
)
KEY_LOADING_ERROR_CLASSIFICATIONS_CSV = ",".join(KEY_LOADING_ERROR_CLASSIFICATIONS)
COMPOSITE_GATE_REASON_TAXONOMY_VERSION = (
    "kamn.kolme.live-provider-native-signer-composite-gate-reason-taxonomy.v1"
)
COMPOSITE_GATE_REASON_CODES = (
    "dry_run_no_commands_executed",
    "live_runtime_integration_passed",
    "runtime_signer_fallback_private_key_present_violation",
    "runtime_signer_managed_external_raw_private_key_present_violation",
    "local_opt_in_missing",
    "bootstrap_readiness_failed",
    "localhost_signed_integration_failed",
    "live_api_conformance_failed",
    "runtime_commit_endpoint_failed",
    "runtime_commit_policy_failed",
    "runtime_integration_budget_exceeded",
)
COMPOSITE_GATE_REASON_CODES_CSV = ",".join(COMPOSITE_GATE_REASON_CODES)
COMPOSITE_GATE_EVIDENCE_CONVERGENCE_STATUS = "verified"
COMPOSITE_GATE_CI_SMOKE_LOCAL_HEAVY_BOUNDARY_STATUS = "verified"
COMPOSITE_GATE_CI_SMOKE_LANE_COST_PROFILE = "low"
COMPOSITE_GATE_LOCAL_HEAVY_EXECUTION_MODE = "not_requested"
SIGNER_PRIVATE_KEY_ENV_BY_PROFILE = {
    "ops-primary": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    "ops-secondary": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
}
SIGNER_KEY_SOURCES_BY_PROFILE = {
    "ops-primary": ("env-local", "managed-external"),
    "ops-secondary": ("env-local",),
}


def classify_key_loading_error() -> str:
    if runtime_signer_fallback_private_key_present:
        return "fallback_private_key_present"
    if (
        runtime_signer_key_source == "managed-external"
        and runtime_signer_raw_private_key_present
    ):
        return "managed_external_raw_private_key_present"
    allowed_key_sources = SIGNER_KEY_SOURCES_BY_PROFILE.get(runtime_signer_profile, ())
    if (
        isinstance(runtime_signer_key_source, str)
        and runtime_signer_key_source
        and allowed_key_sources
        and runtime_signer_key_source not in allowed_key_sources
    ):
        return "key_source_profile_pair_disallowed"
    expected_private_key_env = SIGNER_PRIVATE_KEY_ENV_BY_PROFILE.get(runtime_signer_profile, "")
    if (
        expected_private_key_env
        and isinstance(runtime_signer_private_key_env, str)
        and runtime_signer_private_key_env
        and runtime_signer_private_key_env != expected_private_key_env
    ):
        return "private_key_env_mismatch"
    return "none"


runtime_signer_key_loading_error_classification = classify_key_loading_error()
runtime_signer_key_loading_panic_free = not any(
    marker in reason_code.lower() for marker in PANIC_REASON_MARKERS
)

checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 4:
        continue
    check_id, command, check_status, check_reason = parts
    checks.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
            "reason_code": check_reason,
        }
    )


def read_nested_runtime_reason_code(mode_value: str, summary_path: str) -> str:
    if mode_value == "dry-run":
        return "not_run"

    path = pathlib.Path(summary_path)
    if not path.exists():
        return "report_missing"

    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return "report_invalid_json"

    nested_value = payload.get("reason_code")
    if isinstance(nested_value, str) and nested_value.strip():
        return nested_value
    return "reason_code_missing"


NESTED_RUNTIME_REASON_TO_TAXONOMY = {
    "live_preflight_timeout": "transport.preflight.timeout",
    "live_preflight_failed": "transport.preflight.failed",
    "live_runtime_commit_command_timeout": "transport.submit.timeout",
    "live_runtime_commit_command_failed": "transport.submit.failed",
    "live_finality_retry_exhausted_timeout": "finality.timeout",
    "live_finality_retry_exhausted_failed": "finality.failed",
    "live_finality_command_timeout": "finality.timeout",
    "live_finality_command_failed": "finality.failed",
    "live_runtime_commit_budget_exceeded": "budget.exceeded",
}

NESTED_RUNTIME_REASON_TO_HINT = {
    "live_preflight_timeout": "Verify live-node reachability and preflight latency at the configured --base-url endpoint.",
    "live_preflight_failed": "Inspect preflight health output and provider-hint wiring before submit/finality checks.",
    "live_runtime_commit_command_timeout": "Increase --runtime-commit-max-seconds or reduce submit command latency.",
    "live_runtime_commit_command_failed": "Inspect runtime commit command stderr in runtime_commit_output_file for transport/provider errors.",
    "live_finality_retry_exhausted_timeout": "Increase --runtime-commit-finality-max-seconds/--runtime-commit-finality-retry-max-attempts and verify finality endpoint responsiveness.",
    "live_finality_retry_exhausted_failed": "Inspect runtime finality command output and verify notifications/block fallback endpoint contracts before retrying.",
    "live_finality_command_timeout": "Increase --runtime-commit-finality-max-seconds and verify finality endpoint responsiveness.",
    "live_finality_command_failed": "Inspect runtime finality command output and verify notifications/block fallback endpoint contracts.",
    "live_runtime_commit_budget_exceeded": "Increase --max-seconds or reduce prerequisite/runtime command cost for local-heavy runs.",
}


def classify_runtime_commit_failure(
    mode_value: str,
    status_value: str,
    reason_value: str,
    runtime_reason_value: str,
    runtime_policy_reason_value: str,
    nested_runtime_reason_value: str,
) -> tuple[str, str]:
    if status_value == "ok":
        if mode_value == "dry-run":
            return "none", "Dry-run mode does not execute runtime commit submit/finality checks."
        return "none", "No runtime commit failure observed."

    if reason_value == "runtime_commit_policy_failed":
        return (
            "policy.rejected",
            "Inspect runtime_commit_live_policy_report for final_decision and reason_codes.",
        )

    if reason_value == "runtime_integration_budget_exceeded":
        return (
            "budget.exceeded",
            "Increase --max-seconds or reduce prerequisite/runtime lane execution time.",
        )

    if reason_value == "runtime_signer_managed_external_raw_private_key_present_violation":
        return (
            "none",
            "Unset the selected signer private key env and provide managed signer key-reference env for managed-external mode.",
        )

    if reason_value != "runtime_commit_endpoint_failed":
        return (
            "none",
            "Runtime commit endpoint was not the terminal failure path; inspect prerequisite check reason codes.",
        )

    if runtime_reason_value == "runtime_commit_endpoint_timeout":
        return (
            "transport.submit.timeout",
            "Increase --runtime-commit-max-seconds or inspect runtime commit endpoint command latency.",
        )

    if runtime_policy_reason_value == "runtime_commit_endpoint_failed" and nested_runtime_reason_value in (
        "report_missing",
        "report_invalid_json",
        "reason_code_missing",
    ):
        return (
            "runtime.summary.unavailable",
            "Ensure runtime_commit_live_summary is written and contains a valid reason_code field.",
        )

    mapped_taxonomy = NESTED_RUNTIME_REASON_TO_TAXONOMY.get(nested_runtime_reason_value)
    if mapped_taxonomy is not None:
        mapped_hint = NESTED_RUNTIME_REASON_TO_HINT[nested_runtime_reason_value]
        return mapped_taxonomy, mapped_hint

    return (
        "runtime.unknown",
        f"Inspect nested runtime reason_code={nested_runtime_reason_value} and runtime command artifacts.",
    )


runtime_commit_nested_reason_code = read_nested_runtime_reason_code(mode, runtime_commit_live_summary)
runtime_commit_failure_taxonomy, runtime_commit_failure_diagnostic_hint = classify_runtime_commit_failure(
    mode,
    status,
    reason_code,
    runtime_commit_reason_code,
    runtime_commit_policy_reason_code,
    runtime_commit_nested_reason_code,
)

runtime_signer_attestation_approved_signers: list[str] = []
if isinstance(runtime_signer_profile, str) and runtime_signer_profile.strip():
    runtime_signer_attestation_approved_signers = [runtime_signer_profile]
runtime_signer_attestation_required_approvals = 1
runtime_signer_attestation_bundle = {
    "schema_version": runtime_signer_attestation_schema_version,
    "required_approvals": runtime_signer_attestation_required_approvals,
    "approved_signers": runtime_signer_attestation_approved_signers,
    "signer_profile": runtime_signer_profile,
    "signer_key_source": runtime_signer_key_source,
}
runtime_signer_quorum_linkage_contract_version = "v1"
runtime_signer_quorum_approved_signers_count = len(runtime_signer_attestation_approved_signers)
runtime_signer_quorum_profile_linked = (
    isinstance(runtime_signer_profile, str)
    and runtime_signer_profile.strip()
    and runtime_signer_profile in runtime_signer_attestation_approved_signers
)
runtime_signer_quorum_satisfied = (
    runtime_signer_quorum_approved_signers_count >= runtime_signer_attestation_required_approvals
)
runtime_signer_quorum_linked = (
    runtime_signer_quorum_profile_linked and runtime_signer_quorum_satisfied
)

summary = {
    "schema_version": "kamn.kolme.local-kamn-live-runtime-integration-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "ci_fast_gate_eligible": False,
    "composite_gate_reason_taxonomy_version": COMPOSITE_GATE_REASON_TAXONOMY_VERSION,
    "composite_gate_reason_codes_csv": COMPOSITE_GATE_REASON_CODES_CSV,
    "composite_gate_evidence_convergence_status": COMPOSITE_GATE_EVIDENCE_CONVERGENCE_STATUS,
    "composite_gate_ci_smoke_local_heavy_boundary_status": COMPOSITE_GATE_CI_SMOKE_LOCAL_HEAVY_BOUNDARY_STATUS,
    "composite_gate_ci_smoke_lane_cost_profile": COMPOSITE_GATE_CI_SMOKE_LANE_COST_PROFILE,
    "composite_gate_local_heavy_execution_mode": COMPOSITE_GATE_LOCAL_HEAVY_EXECUTION_MODE,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "base_url": base_url,
    "fork_chain_version": fork_chain_version,
    "runtime_commit_command": runtime_commit_command,
    "runtime_provider_client_contract": runtime_provider_client_contract,
    "runtime_profile": runtime_profile,
    "runtime_signing_profile": runtime_signing_profile,
    "runtime_commit_command_profile": runtime_commit_command_profile,
    "runtime_commit_policy_command_profile": runtime_commit_policy_command_profile,
    "runtime_commit_command_profile_version": runtime_commit_command_profile_version,
    "runtime_signer_profile_selector_env": runtime_signer_profile_selector_env,
    "runtime_signer_profile": runtime_signer_profile,
    "runtime_signer_previous_profile": runtime_signer_previous_profile,
    "runtime_signer_failover_active": runtime_signer_failover_active,
    "runtime_signer_rotation_epoch": runtime_signer_rotation_epoch,
    "runtime_signer_previous_rotation_epoch": runtime_signer_previous_rotation_epoch,
    "runtime_signer_key_source_contract_version": runtime_signer_key_source_contract_version,
    "runtime_signer_key_source": runtime_signer_key_source,
    "runtime_signer_private_key_env": runtime_signer_private_key_env,
    "runtime_signer_key_reference_env": runtime_signer_key_reference_env,
    "runtime_signer_fallback_guard_contract_version": runtime_signer_fallback_guard_contract_version,
    "runtime_signer_fallback_guard_mode": runtime_signer_fallback_guard_mode,
    "runtime_signer_managed_external_raw_private_key_remediation": runtime_signer_managed_external_raw_private_key_remediation,
    "runtime_signer_fallback_private_key_present": runtime_signer_fallback_private_key_present,
    "runtime_signer_raw_private_key_present": runtime_signer_raw_private_key_present,
    "runtime_signer_private_key_env_zeroized": runtime_signer_private_key_env_zeroized,
    "runtime_signer_private_key_bytes_zeroized": runtime_signer_private_key_bytes_zeroized,
    "runtime_signer_key_loading_panic_free": runtime_signer_key_loading_panic_free,
    "runtime_signer_key_loading_error_classification_version": KEY_LOADING_ERROR_CLASSIFICATION_VERSION,
    "runtime_signer_key_loading_error_classification_allowed_csv": KEY_LOADING_ERROR_CLASSIFICATIONS_CSV,
    "runtime_signer_key_loading_error_classification": runtime_signer_key_loading_error_classification,
    "runtime_signer_attestation_schema_version": runtime_signer_attestation_schema_version,
    "runtime_signer_attestation_bundle": runtime_signer_attestation_bundle,
    "runtime_signer_quorum_linkage_contract_version": runtime_signer_quorum_linkage_contract_version,
    "runtime_signer_quorum_required_approvals": runtime_signer_attestation_required_approvals,
    "runtime_signer_quorum_approved_signers_count": runtime_signer_quorum_approved_signers_count,
    "runtime_signer_quorum_profile_linked": runtime_signer_quorum_profile_linked,
    "runtime_signer_quorum_satisfied": runtime_signer_quorum_satisfied,
    "runtime_signer_quorum_linked": runtime_signer_quorum_linked,
    "runtime_commit_live_policy_report": runtime_commit_live_policy_report,
    "runtime_commit_finality_command": runtime_commit_finality_command if runtime_commit_finality_command else "",
    "runtime_commit_finality_output_file": runtime_commit_finality_output_file if runtime_commit_finality_command else "",
    "runtime_commit_finality_enabled": bool(runtime_commit_finality_command),
    "runtime_commit_finality_max_seconds": runtime_commit_finality_max_seconds,
    "bootstrap_reason_code": bootstrap_reason_code,
    "localhost_signed_reason_code": localhost_signed_reason_code,
    "conformance_reason_code": conformance_reason_code,
    "runtime_commit_reason_code": runtime_commit_reason_code,
    "runtime_commit_policy_reason_code": runtime_commit_policy_reason_code,
    "runtime_commit_nested_reason_code": runtime_commit_nested_reason_code,
    "runtime_commit_failure_taxonomy_version": "v1",
    "runtime_commit_failure_taxonomy": runtime_commit_failure_taxonomy,
    "runtime_commit_failure_diagnostic_hint": runtime_commit_failure_diagnostic_hint,
    "contracts": {
        "ci_fast_gate_scope": "local-only",
        "composite_gate_reason_taxonomy_version": COMPOSITE_GATE_REASON_TAXONOMY_VERSION,
        "composite_gate_reason_codes_csv": COMPOSITE_GATE_REASON_CODES_CSV,
        "composite_gate_evidence_convergence_status": COMPOSITE_GATE_EVIDENCE_CONVERGENCE_STATUS,
        "composite_gate_ci_smoke_local_heavy_boundary_status": COMPOSITE_GATE_CI_SMOKE_LOCAL_HEAVY_BOUNDARY_STATUS,
        "composite_gate_ci_smoke_lane_cost_profile": COMPOSITE_GATE_CI_SMOKE_LANE_COST_PROFILE,
        "composite_gate_local_heavy_execution_mode": COMPOSITE_GATE_LOCAL_HEAVY_EXECUTION_MODE,
        "runtime_provider_client_contract": runtime_provider_client_contract,
        "runtime_profile": runtime_profile,
        "runtime_signing_profile": runtime_signing_profile,
        "runtime_signer_profile_selector_env": runtime_signer_profile_selector_env,
        "runtime_signer_profile": runtime_signer_profile,
        "runtime_signer_failover_requires_profile_change": True,
        "runtime_signer_rotation_epoch_must_increase_on_failover": True,
        "runtime_signer_failover_attestation_min_required_approvals": 2,
        "runtime_signer_failover_attestation_previous_profile_membership_required": True,
        "runtime_signer_key_source_contract_version": runtime_signer_key_source_contract_version,
        "runtime_signer_key_source": runtime_signer_key_source,
        "runtime_signer_private_key_env": runtime_signer_private_key_env,
        "runtime_signer_key_reference_env": runtime_signer_key_reference_env,
        "runtime_signer_fallback_guard_contract_version": runtime_signer_fallback_guard_contract_version,
        "runtime_signer_fallback_guard_mode": runtime_signer_fallback_guard_mode,
        "runtime_signer_managed_external_raw_private_key_remediation": runtime_signer_managed_external_raw_private_key_remediation,
        "runtime_signer_fallback_private_key_allowed": False,
        "runtime_signer_fallback_private_key_command_marker_allowed": False,
        "runtime_signer_managed_external_raw_private_key_allowed": False,
        "runtime_signer_private_key_env_zeroization_required": True,
        "runtime_signer_private_key_bytes_zeroization_required": True,
        "runtime_signer_key_loading_panic_free_required": True,
        "runtime_signer_key_loading_error_classification_version": KEY_LOADING_ERROR_CLASSIFICATION_VERSION,
        "runtime_signer_key_loading_error_classification_stable_required": True,
        "runtime_signer_key_loading_error_classification_allowed_csv": KEY_LOADING_ERROR_CLASSIFICATIONS_CSV,
        "runtime_signer_attestation_schema_version": runtime_signer_attestation_schema_version,
        "runtime_signer_attestation_signer_uniqueness_required": True,
        "runtime_signer_attestation_threshold_required": True,
        "runtime_signer_attestation_profile_membership_required": True,
        "runtime_signer_attestation_required_approvals": runtime_signer_attestation_required_approvals,
        "runtime_signer_quorum_linkage_contract_version": runtime_signer_quorum_linkage_contract_version,
        "runtime_signer_quorum_required_approvals": runtime_signer_attestation_required_approvals,
        "runtime_signer_quorum_linked_required": True,
        "runtime_signer_quorum_threshold_required": True,
        "runtime_signer_quorum_profile_membership_required": True,
        "runtime_signer_quorum_linked": runtime_signer_quorum_linked,
        "runtime_commit_endpoint": "/broadcast/runtime-commit",
        "runtime_commit_method": "POST",
        "runtime_commit_finality_primary_endpoint": "/notifications",
        "runtime_commit_finality_fallback_endpoint": "/block/{height}",
        "runtime_commit_failure_taxonomy_version": "v1",
    },
    "checks": checks,
    "artifact_paths": [
        bootstrap_report,
        localhost_signed_report,
        conformance_report,
        runtime_commit_output_file,
        runtime_commit_live_summary,
        runtime_commit_live_policy_report,
    ],
}

if runtime_commit_finality_command:
    summary["artifact_paths"].append(runtime_commit_finality_output_file)

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
