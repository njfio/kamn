# Signer Lifecycle and Preflight Policy

This document defines the `kolme-live` signer lifecycle contract enforced by `kamn-node` at runtime startup and at direct-sign payload construction.

## Scope

- Runtime mode: `kolme-live`
- Signer profiles: `ops-primary`, `ops-secondary`
- Key sources:
- `env-local`
- `managed-external`

## Preflight Gate

`kamn-node` performs signer preflight in three layers:

1. Contract-policy gates in `main.rs`
- Rejects legacy local signer path unless strict contracts are enabled, local override is explicit, or test build path is active.
- Rejects `env-local` key source in production-targeted strict mode.

2. Startup signer preflight in `signer.rs`
- Resolves signer selection from declared profile/key-source inputs.
- Evaluates rotation/failover/quorum readiness.
- Rejects fallback signer secret path (`KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK`).
- For strict `env-local` signer contracts, rejects dual-secret sources by requiring the
  non-selected profile private-key env marker to remain unset.
- For `managed-external`, verifies:
- managed signer command marker exists
- managed signer public key marker exists and is valid secp256k1 compressed key material
- managed signer key reference marker exists and is valid `secure:<provider>:role-operator/...`
- raw private-key env path for the same profile remains unset

3. Per-request signing checks
- Managed-external signature provenance is verified before payload emission.
- Managed-external failures fail closed before nonce/network submit when markers are missing.

## Deterministic Reason Codes

Primary fail-closed signer reason codes include:

- `legacy_local_signer_path_forbidden`
- `legacy_local_signer_path_override_invalid`
- `production_signer_key_source_env_local_forbidden`
- `fallback_signer_secret_present_violation`
- `runtime_signer_profile_selector_mismatch`
- `signer_secret_source_precedence_violation`
- `managed_signer_backend_required_missing`
- `managed_signer_backend_required_invalid`
- `managed_signer_key_reference_missing`
- `managed_signer_key_reference_invalid`
- `managed_signer_key_reference_role_invalid`
- `managed_signer_public_key_marker_missing`
- `managed_signer_public_key_marker_invalid`
- `managed_signer_raw_private_key_forbidden`
- `runtime_signer_rotation_epoch_stale`
- `runtime_signer_key_source_profile_pair_disallowed`
- `runtime_signer_attestation_quorum_shortfall`

## Operational Remediation

- If managed-external marker checks fail:
- set `KAMN_KOLME_LIVE_MANAGED_SIGNER_COMMAND`
- set profile-specific public key marker:
- `KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX` for `ops-primary`
- `KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY` for `ops-secondary`
- set profile-specific managed key reference marker:
- `KAMN_KOLME_LIVE_SIGNER_KEY_REF` for `ops-primary`
- `KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY` for `ops-secondary`
- unset profile-local raw private key marker when using `managed-external`
- ensure fallback private key env marker is unset

- If strict `env-local` signer-source checks fail:
- unset non-selected profile private key marker:
- `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY` when selecting `ops-primary`
- `KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX` when selecting `ops-secondary`

- If failover/rotation checks fail:
- increment `KAMN_KOLME_LIVE_SIGNER_ROTATION_EPOCH` relative to previous epoch
- ensure `KAMN_KOLME_LIVE_SIGNER_QUORUM_APPROVED_SIGNERS` includes both current and previous profile during failover
- ensure `KAMN_KOLME_LIVE_SIGNER_QUORUM_REQUIRED_APPROVALS` is at least `2` during failover

## Testing Coverage

- Unit: signer preflight defaults and parser invariants.
- Functional: failover/quorum and key-source policy enforcement.
- Integration: runtime `kolme-live` fail-closed behavior before network paths for policy violations.
- Regression: deterministic reason-code assertions for signer provenance and fallback paths.
