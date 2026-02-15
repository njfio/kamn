#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

payload_path = pathlib.Path(sys.argv[1])
expected_schema = sys.argv[2]
required_approvals = int(sys.argv[3])
received_approvals = int(sys.argv[4])
custody_sha256 = sys.argv[5]
runtime_signer_profile = sys.argv[6]

result = {
    "schema_valid": False,
    "approval_count": 0,
    "signers_unique": False,
    "matches_threshold": False,
    "custody_sha256_match": False,
    "signer_roles_present": False,
    "signer_roles_valid": False,
    "rotation_metadata_present": False,
    "rotation_metadata_valid": False,
    "profile_approved": False,
    "approved_signers_csv": "",
    "reason_code": "quorum_evidence_schema_invalid",
}

try:
    payload = json.loads(payload_path.read_text(encoding="utf-8"))
except Exception:
    payload = None

if isinstance(payload, dict):
    schema_valid = payload.get("schema_version") == expected_schema
    approved_signers_raw = payload.get("approved_signers")
    normalized_signers: list[str] = []
    if isinstance(approved_signers_raw, list):
        for item in approved_signers_raw:
            if isinstance(item, str) and item.strip():
                normalized_signers.append(item.strip())
    approval_count = len(normalized_signers)
    signers_unique = approval_count > 0 and len(set(normalized_signers)) == approval_count
    required_matches = payload.get("required_approvals") == required_approvals
    received_matches = payload.get("received_approvals") == received_approvals
    matches_threshold = (
        required_matches
        and received_matches
        and approval_count == received_approvals
        and received_approvals >= required_approvals
    )
    signer_roles_raw = payload.get("signer_roles")
    signer_roles_present = False
    signer_roles_valid = False
    if isinstance(signer_roles_raw, dict):
        signer_roles_present = approval_count > 0 and all(
            signer in signer_roles_raw for signer in normalized_signers
        )
        if signer_roles_present:
            signer_roles_valid = True
            for signer in normalized_signers:
                role_value = signer_roles_raw.get(signer)
                if role_value not in ("primary", "secondary"):
                    signer_roles_valid = False
                    break

    signer_rotation_epochs_raw = payload.get("signer_rotation_epochs")
    rotation_metadata_present = False
    rotation_metadata_valid = False
    if isinstance(signer_rotation_epochs_raw, dict):
        rotation_metadata_present = approval_count > 0 and all(
            signer in signer_rotation_epochs_raw for signer in normalized_signers
        )
        if rotation_metadata_present:
            rotation_metadata_valid = True
            for signer in normalized_signers:
                epoch_value = signer_rotation_epochs_raw.get(signer)
                if not isinstance(epoch_value, int) or epoch_value <= 0:
                    rotation_metadata_valid = False
                    break

    custody_field = payload.get("custody_evidence_sha256")
    custody_match = isinstance(custody_field, str) and custody_field == custody_sha256
    profile_approved = runtime_signer_profile in normalized_signers

    result["schema_valid"] = schema_valid
    result["approval_count"] = approval_count
    result["signers_unique"] = signers_unique
    result["matches_threshold"] = matches_threshold
    result["custody_sha256_match"] = custody_match
    result["signer_roles_present"] = signer_roles_present
    result["signer_roles_valid"] = signer_roles_valid
    result["rotation_metadata_present"] = rotation_metadata_present
    result["rotation_metadata_valid"] = rotation_metadata_valid
    result["profile_approved"] = profile_approved
    result["approved_signers_csv"] = ",".join(normalized_signers)

    if not schema_valid:
        result["reason_code"] = "runtime_signer_attestation_schema_invalid"
    elif not signers_unique:
        result["reason_code"] = "runtime_signer_attestation_approved_signers_not_unique"
    elif not signer_roles_present:
        result["reason_code"] = "quorum_evidence_signer_roles_missing"
    elif not signer_roles_valid:
        result["reason_code"] = "quorum_evidence_signer_roles_invalid"
    elif not rotation_metadata_present:
        result["reason_code"] = "quorum_evidence_rotation_metadata_missing"
    elif not rotation_metadata_valid:
        result["reason_code"] = "quorum_evidence_rotation_metadata_invalid"
    elif not matches_threshold:
        result["reason_code"] = "runtime_signer_attestation_quorum_shortfall"
    elif not profile_approved:
        result["reason_code"] = "runtime_signer_attestation_profile_not_approved"
    elif not custody_match:
        result["reason_code"] = "quorum_evidence_custody_sha256_mismatch"
    else:
        result["reason_code"] = "ok"

for key in (
    "schema_valid",
    "approval_count",
    "signers_unique",
    "matches_threshold",
    "custody_sha256_match",
    "signer_roles_present",
    "signer_roles_valid",
    "rotation_metadata_present",
    "rotation_metadata_valid",
    "profile_approved",
    "approved_signers_csv",
    "reason_code",
):
    print(f"{key}={result[key]}")
