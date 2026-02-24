# Transaction Guards (Issue #78)

This document defines transaction guards in `kamn-core` with cryptographic signature validation by default and explicit legacy compatibility gating.

## Invariants Enforced
- Non-empty transaction envelope fields (`id`, `sender`, `payload`, `state_hash`, `signature`).
- Positive nonce values.
- Per-sender nonce sequencing (first nonce `1`, then increment by `1`).
- State-hash match against the currently expected chain/app state hash.
- Cryptographic signature integrity check (sender + nonce + state hash + payload binding).
- Duplicate transaction ID rejection.

## Components
- `BaselineTransaction`: canonical envelope used by smoke/runtime scaffolding.
- `TransactionGuards`: stateful guard evaluator that validates and records transaction invariants.
- `TransactionGuardError`: explicit typed failures for deterministic operator/agent handling.
- `RoleSmokeNetwork`: integrates guard checks on `submit_transaction(...)` and advances state hash on `produce_block(...)`.
- `INVARIANT_CATALOG`: canonical invariant IDs and failure-code taxonomy mapping (`docs/foundation/invariants.md`).

## Canonical Signature Profile
- Signature profile is shared between `transaction` and `signer_backend` paths.
- Default signature profile is service-auth cryptographic profile (`secp256k1:baseline-v2`):
  - `sig:secp256k1:baseline-v2:<recovery_id>:<signature_hex>`
  - verification uses `service_auth_verify_with_public_key_hex(...)`
- Public key resolution for verification:
  - `KAMN_SIGNER_PUBLIC_KEY_HEX`
  - `KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX`
  - derived from `KAMN_SIGNER_PRIVATE_KEY_HEX` or `KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX` when explicit public key is absent.
- Legacy baseline compatibility helper remains available for migration fixtures:
  - `baseline_signature_for_fields(...)`
  - `signature_profile_compatibility_fixtures_for_fields(...)`
  - fixture IDs include `legacy-unversioned`, `baseline-v0`, and `unknown-algorithm+baseline-v1`.
- Legacy baseline-v1 (`sig:deterministic-v1:baseline-v1:<sender>:<nonce>:<state_hash>:<payload_len>`) is accepted only when `KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1=1`.
- signature-profile drift between transaction and signer paths is rejected (`Regression: #400`).
- non-versioned legacy signature profile is rejected (`Regression: #404`).
- algorithm/profile metadata drift or downgrade is rejected (`Regression: #677`).
- default fail-closed mode keeps legacy compatibility fixtures rejected unless explicit compatibility switch is enabled (`Regression: #5897`).

## Signer Fallback Policy Integration
- signer keys support role-scoped secure references: `secure:aws-kms:role-<operator|admin|treasury|auditor>/<key-ref>`.
- sender role prefixes map to signing roles and are validated against key role before signing.
- role mismatch is rejected through typed `KeyRoleMismatch`.
- secure fallback to local signing is permitted only for `operator` role keys.
- privileged roles (`admin`, `treasury`, `auditor`) reject fallback via `FallbackDeniedByRolePolicy` (`Regression: #619`).

## Validation
Run from repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```
