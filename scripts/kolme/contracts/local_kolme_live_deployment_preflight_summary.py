#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
elapsed_seconds = int(sys.argv[5])
max_seconds = int(sys.argv[6])
budget_status = sys.argv[7]
runtime_mode = sys.argv[8]
signer_profile_selector_env = sys.argv[9]
signer_profile = sys.argv[10]
signer_private_key_env = sys.argv[11]
fallback_signer_private_key_env = sys.argv[12]
signer_secret_present = sys.argv[13] == "true"
fallback_signer_secret_present = sys.argv[14] == "true"
signer_secret_hex_valid = sys.argv[15] == "true"
checks_path = pathlib.Path(sys.argv[16])
required_runtime_mode = sys.argv[17]
primary_signer_profile = sys.argv[18]
secondary_signer_profile = sys.argv[19]
primary_signer_secret_env = sys.argv[20]
secondary_signer_secret_env = sys.argv[21]
required_secret_hex_length = int(sys.argv[22])
required_approvals = int(sys.argv[23])
received_approvals = int(sys.argv[24])
quorum_evidence_file = sys.argv[25]
quorum_evidence_present = sys.argv[26] == "true"
quorum_evidence_sha256 = sys.argv[27]
quorum_evidence_sha256_valid = sys.argv[28] == "true"
quorum_evidence_schema_valid = sys.argv[29] == "true"
quorum_evidence_approval_count = int(sys.argv[30])
quorum_evidence_signers_unique = sys.argv[31] == "true"
quorum_evidence_matches_threshold = sys.argv[32] == "true"
quorum_evidence_custody_sha256_match = sys.argv[33] == "true"
quorum_evidence_signer_roles_present = sys.argv[34] == "true"
quorum_evidence_signer_roles_valid = sys.argv[35] == "true"
quorum_evidence_rotation_metadata_present = sys.argv[36] == "true"
quorum_evidence_rotation_metadata_valid = sys.argv[37] == "true"
quorum_evidence_schema_version = sys.argv[38]
custody_evidence_file = sys.argv[39]
custody_evidence_present = sys.argv[40] == "true"
custody_evidence_sha256 = sys.argv[41]
custody_evidence_sha256_valid = sys.argv[42] == "true"
signer_provenance_file = sys.argv[43]
signer_provenance_present = sys.argv[44] == "true"
signer_provenance_sha256 = sys.argv[45]
signer_provenance_sha256_valid = sys.argv[46] == "true"
signer_key_source_contract_version = sys.argv[47]
signer_key_source = sys.argv[48]
signer_key_source_contract_version_supported = sys.argv[49]
signer_rotation_epoch = int(sys.argv[50])
signer_previous_rotation_epoch = int(sys.argv[51])
signer_rotation_freshness_max_delta = int(sys.argv[52])
signer_rotation_delta_epochs = int(sys.argv[53])
signer_rotation_fresh = sys.argv[54] == "true"
runtime_signer_attestation_schema_version = sys.argv[55]
runtime_signer_attestation_approved_signers_csv = sys.argv[56]
runtime_signer_attestation_profile_approved = sys.argv[57] == "true"
runtime_signer_drift_telemetry_schema_version = sys.argv[58]
runtime_signer_drift_thresholds_schema_version = sys.argv[59]

runtime_signer_attestation_approved_signers: list[str] = [
    entry.strip()
    for entry in runtime_signer_attestation_approved_signers_csv.split(",")
    if entry.strip()
]
if mode == "dry-run" and not runtime_signer_attestation_approved_signers:
    runtime_signer_attestation_approved_signers = [primary_signer_profile, secondary_signer_profile]

runtime_signer_attestation_profile_approved = (
    isinstance(signer_profile, str) and signer_profile in runtime_signer_attestation_approved_signers
)
signer_profile_class = "production" if signer_profile in (primary_signer_profile, secondary_signer_profile) else "unknown"
fallback_signer_secret_remediation = f"unset {fallback_signer_private_key_env}"

runtime_signer_attestation_bundle = {
    "schema_version": runtime_signer_attestation_schema_version,
    "required_approvals": required_approvals,
    "approved_signers": runtime_signer_attestation_approved_signers,
    "signer_profile": signer_profile,
    "signer_key_source": signer_key_source,
}
runtime_signer_drift_telemetry = {
    "schema_version": runtime_signer_drift_telemetry_schema_version,
    "signer_rotation_epoch": signer_rotation_epoch,
    "signer_previous_rotation_epoch": signer_previous_rotation_epoch,
    "signer_rotation_delta_epochs": signer_rotation_delta_epochs,
    "signer_rotation_freshness_max_delta": signer_rotation_freshness_max_delta,
    "signer_rotation_stale": signer_rotation_delta_epochs > signer_rotation_freshness_max_delta,
    "required_approvals": required_approvals,
    "received_approvals": received_approvals,
    "quorum_shortfall": received_approvals < required_approvals,
}
runtime_signer_drift_thresholds_bundle = {
    "schema_version": runtime_signer_drift_thresholds_schema_version,
    "rotation_warn_delta_epochs": max(0, signer_rotation_freshness_max_delta - 1),
    "rotation_fail_delta_epochs": signer_rotation_freshness_max_delta,
    "quorum_warn_shortfall_events": 0,
    "quorum_fail_shortfall_events": 0,
}

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

summary = {
    "schema_version": "kamn.kolme.local-live-deployment-preflight-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": False,
    "ci_fast_gate_eligible": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "runtime_mode": runtime_mode,
    "signer_profile_selector_env": signer_profile_selector_env,
    "signer_profile": signer_profile,
    "signer_profile_class": signer_profile_class,
    "signer_private_key_env": signer_private_key_env,
    "fallback_signer_private_key_env": fallback_signer_private_key_env,
    "fallback_signer_secret_remediation": fallback_signer_secret_remediation,
    "signer_secret_present": signer_secret_present,
    "fallback_signer_secret_present": fallback_signer_secret_present,
    "signer_secret_hex_valid": signer_secret_hex_valid,
    "required_approvals": required_approvals,
    "received_approvals": received_approvals,
    "quorum_evidence_file": quorum_evidence_file,
    "quorum_evidence_present": quorum_evidence_present,
    "quorum_evidence_sha256": quorum_evidence_sha256,
    "quorum_evidence_sha256_valid": quorum_evidence_sha256_valid,
    "quorum_evidence_schema_valid": quorum_evidence_schema_valid,
    "quorum_evidence_approval_count": quorum_evidence_approval_count,
    "quorum_evidence_signers_unique": quorum_evidence_signers_unique,
    "quorum_evidence_matches_threshold": quorum_evidence_matches_threshold,
    "quorum_evidence_custody_sha256_match": quorum_evidence_custody_sha256_match,
    "quorum_evidence_signer_roles_present": quorum_evidence_signer_roles_present,
    "quorum_evidence_signer_roles_valid": quorum_evidence_signer_roles_valid,
    "quorum_evidence_rotation_metadata_present": quorum_evidence_rotation_metadata_present,
    "quorum_evidence_rotation_metadata_valid": quorum_evidence_rotation_metadata_valid,
    "runtime_signer_attestation_schema_version": runtime_signer_attestation_schema_version,
    "runtime_signer_attestation_bundle": runtime_signer_attestation_bundle,
    "runtime_signer_attestation_profile_approved": runtime_signer_attestation_profile_approved,
    "runtime_signer_drift_telemetry_schema_version": runtime_signer_drift_telemetry_schema_version,
    "runtime_signer_drift_telemetry": runtime_signer_drift_telemetry,
    "runtime_signer_drift_thresholds_schema_version": runtime_signer_drift_thresholds_schema_version,
    "runtime_signer_drift_thresholds_bundle": runtime_signer_drift_thresholds_bundle,
    "custody_evidence_file": custody_evidence_file,
    "custody_evidence_present": custody_evidence_present,
    "custody_evidence_sha256": custody_evidence_sha256,
    "custody_evidence_sha256_valid": custody_evidence_sha256_valid,
    "signer_provenance_file": signer_provenance_file,
    "signer_provenance_present": signer_provenance_present,
    "signer_provenance_sha256": signer_provenance_sha256,
    "signer_provenance_sha256_valid": signer_provenance_sha256_valid,
    "signer_key_source_contract_version": signer_key_source_contract_version,
    "signer_key_source": signer_key_source,
    "signer_rotation_epoch": signer_rotation_epoch,
    "signer_previous_rotation_epoch": signer_previous_rotation_epoch,
    "signer_rotation_freshness_max_delta": signer_rotation_freshness_max_delta,
    "signer_rotation_delta_epochs": signer_rotation_delta_epochs,
    "signer_rotation_fresh": signer_rotation_fresh,
    "contracts": {
        "ci_fast_gate_scope": "ci-fast-gate",
        "required_runtime_mode": required_runtime_mode,
        "signer_profile_selector_env": signer_profile_selector_env,
        "supported_signer_profiles": [primary_signer_profile, secondary_signer_profile],
        "primary_signer_secret_env": primary_signer_secret_env,
        "secondary_signer_secret_env": secondary_signer_secret_env,
        "fallback_signer_secret_env": fallback_signer_private_key_env,
        "fallback_signer_secret_rejected_profile_class": "production",
        "fallback_signer_secret_rejected_profiles": [primary_signer_profile, secondary_signer_profile],
        "fallback_signer_secret_remediation": fallback_signer_secret_remediation,
        "fallback_signer_secret_rejection_reason_code": "fallback_signer_secret_present_violation",
        "fallback_signer_secret_checkpoint_reason_code": "checkpoint_failed_fallback_private_key_contract",
        "fallback_private_key_path_allowed": False,
        "required_secret_hex_length": required_secret_hex_length,
        "secret_source": "env",
        "approval_quorum_minimum": 2,
        "approval_quorum_required": required_approvals,
        "approval_quorum_source": "local-operator-attestations",
        "quorum_evidence_required": True,
        "quorum_evidence_sha256_required": True,
        "quorum_evidence_schema_version": quorum_evidence_schema_version,
        "quorum_evidence_signer_uniqueness_required": True,
        "quorum_evidence_custody_sha256_match_required": True,
        "quorum_evidence_signer_roles_required": True,
        "quorum_evidence_signer_roles_allowed": ["primary", "secondary"],
        "quorum_evidence_rotation_metadata_required": True,
        "quorum_evidence_rotation_metadata_positive_epochs_required": True,
        "quorum_evidence_source": "operator-attestation-bundle",
        "runtime_signer_attestation_schema_version": runtime_signer_attestation_schema_version,
        "runtime_signer_attestation_signer_uniqueness_required": True,
        "runtime_signer_attestation_threshold_required": True,
        "runtime_signer_attestation_profile_membership_required": True,
        "runtime_signer_attestation_required_approvals": required_approvals,
        "runtime_signer_drift_telemetry_required": True,
        "runtime_signer_drift_telemetry_schema_version": runtime_signer_drift_telemetry_schema_version,
        "runtime_signer_drift_telemetry_rotation_delta_match_required": True,
        "runtime_signer_drift_telemetry_stale_flag_match_required": True,
        "runtime_signer_drift_telemetry_quorum_flag_match_required": True,
        "runtime_signer_drift_telemetry_approval_counts_match_required": True,
        "runtime_signer_drift_thresholds_required": True,
        "runtime_signer_drift_thresholds_schema_version": runtime_signer_drift_thresholds_schema_version,
        "runtime_signer_drift_thresholds_rotation_warn_lte_fail_required": True,
        "runtime_signer_drift_thresholds_quorum_warn_lte_fail_required": True,
        "runtime_signer_drift_admission_matrix_required": True,
        "runtime_signer_drift_admission_matrix_decision_values": ["GO", "WARN", "NO-GO"],
        "custody_evidence_required": True,
        "custody_evidence_sha256_required": True,
        "signer_provenance_required": True,
        "signer_provenance_sha256_required": True,
        "signer_key_source_contract_version": signer_key_source_contract_version_supported,
        "signer_key_source": signer_key_source,
        "required_signer_key_source_for_production": "managed-external",
        "signer_key_source_production_requirement_reason_code": "signer_key_source_production_managed_external_required",
        "signer_key_source_allowed_for_ops_primary": ["managed-external"],
        "signer_key_source_allowed_for_ops_secondary": ["managed-external"],
        "signer_rotation_freshness_max_delta": signer_rotation_freshness_max_delta,
        "signer_rotation_stale_rejected": True,
    },
    "checks": checks,
    "artifact_paths": [],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
