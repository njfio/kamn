# Key Management Policy

## Explicit Runtime Key-Source and Fallback Rejection (Issue #4356)

Local KAMN live runtime integration evidence must include explicit signer key-source markers and fail closed when fallback key paths are reachable.

- Required summary/contract markers:
  - `runtime_signer_key_source_contract_version=v1`
  - `runtime_signer_key_source=env-local|managed-external`
  - `contracts.runtime_signer_key_source_contract_version=v1`
  - `contracts.runtime_signer_key_source=env-local|managed-external`
- Required runtime command marker:
  - `KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE=<env-local|managed-external>`
- Deterministic key-source taxonomy outputs:
  - `key_source_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-key-source-reason-taxonomy.v1`
  - `key_source_reason_codes_csv=runtime_signer_key_source_contract_version_missing,runtime_signer_key_source_contract_version_mismatch,runtime_signer_key_source_contract_version_contract_mismatch,runtime_signer_key_source_missing,runtime_signer_key_source_invalid,runtime_signer_key_source_profile_pair_disallowed,runtime_signer_key_source_contract_mismatch,runtime_commit_signer_key_source_marker_missing,runtime_commit_fallback_private_key_command_marker_detected,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation`
  - `key_source_reason_codes_value=none|<csv>`
- Fail-closed key-source/fallback reasons:
  - `runtime_signer_key_source_contract_version_missing`
  - `runtime_signer_key_source_contract_version_contract_mismatch`
  - `runtime_commit_signer_key_source_marker_missing`
  - `runtime_commit_fallback_private_key_command_marker_detected`
  - `runtime_signer_fallback_private_key_present_violation`

Policy checker reference:

```bash
python3 scripts/kolme/check_local_kamn_live_runtime_integration_policy.py \
  --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json
```

## Rotation Preflight Evidence Matrix (Issue #4358)

Deployment preflight checks for live signer rotation must emit deterministic key-policy/rotation reason mapping markers for rotate-ready and rotate-blocked promotion gates.

- Required taxonomy outputs:
  - `rotation_preflight_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v1`
  - `rotation_preflight_reason_codes_csv=signer_key_source_contract_version_mismatch,signer_key_source_invalid,signer_key_source_production_managed_external_required,signer_quorum_minimum_not_met,signer_rotation_epoch_stale,signer_rotation_rehearsal_drift_detected,signer_rotation_promotion_stalled,fallback_signer_secret_present_violation,fallback_signer_secret_checkpoint_reason_mismatch,fallback_signer_secret_remediation_missing,quorum_evidence_missing,quorum_evidence_rotation_metadata_missing,quorum_evidence_rotation_metadata_invalid,runtime_signer_attestation_quorum_shortfall,runtime_signer_attestation_profile_not_approved,runtime_signer_drift_telemetry_missing,runtime_signer_drift_telemetry_rotation_delta_invalid,runtime_signer_drift_matrix_inputs_invalid,runtime_signer_drift_rotation_fail_threshold_exceeded,runtime_signer_drift_quorum_fail_threshold_exceeded,custody_continuity_bypass_detected`
  - `rotation_preflight_reason_codes_value=none|<csv>`
- High-signal fail-closed rotation/key-policy reasons:
  - `signer_rotation_rehearsal_drift_detected`
  - `signer_key_source_production_managed_external_required`
  - `runtime_signer_drift_rotation_fail_threshold_exceeded`
  - `runtime_signer_drift_quorum_fail_threshold_exceeded`

Policy checker reference:

```bash
python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py \
  --report-file /tmp/kolme-local-live-deployment-preflight-summary.json \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json /tmp/kolme-local-live-deployment-preflight-policy.json
```

## Native secp256k1 Signing Enforcement (Issue #4373)

Production Kolme live-runtime flows must use native secp256k1 signing evidence and fail closed on simulated profiles.

- Required summary markers:
  - `runtime_signing_profile_contract_version=v1`
  - `runtime_signing_profile=kolme-fork-secp256k1-v1`
- Required command marker in run-mode evidence:
  - `KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1`
- Fail-closed reasons:
  - `runtime_commit_native_signing_profile_marker_missing`
  - `runtime_commit_simulated_signing_profile_detected`
  - `runtime_signing_profile_missing`
  - `runtime_signing_profile_mismatch`

Policy checker reference:

```bash
python3 scripts/kolme/check_local_signed_to_kolme_demo_policy.py \
  --report-file /tmp/kolme-local-signed-to-kolme-demo-summary.json \
  --expected-final-decision GO \
  --ci-fast-gate PASS \
  --output-json /tmp/kolme-local-signed-to-kolme-demo-policy.json
```

Deterministic taxonomy outputs:

- `native_signer_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1`
- `native_signer_reason_codes_csv=runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch`
- `native_signer_reason_codes_value=none|runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch`
