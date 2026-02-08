# Transaction Guards (Issue #78)

This document defines the baseline deterministic transaction guards implemented in `kamn-core`.

## Invariants Enforced
- Non-empty transaction envelope fields (`id`, `sender`, `payload`, `state_hash`, `signature`).
- Positive nonce values.
- Per-sender nonce sequencing (first nonce `1`, then increment by `1`).
- State-hash match against the currently expected chain/app state hash.
- Deterministic signature integrity check (explicit algorithm/profile metadata contract).
- Duplicate transaction ID rejection.

## Components
- `BaselineTransaction`: canonical envelope used by smoke/runtime scaffolding.
- `TransactionGuards`: stateful guard evaluator that validates and records transaction invariants.
- `TransactionGuardError`: explicit typed failures for deterministic operator/agent handling.
- `RoleSmokeNetwork`: integrates guard checks on `submit_transaction(...)` and advances state hash on `produce_block(...)`.
- `INVARIANT_CATALOG`: canonical invariant IDs and failure-code taxonomy mapping (`docs/foundation/invariants.md`).

## Canonical Signature Profile
- `baseline_signature_for_fields(...)` is the canonical baseline signing profile helper.
- Signature profile is shared between `transaction` and `signer_backend` paths.
- baseline signature algorithm: `ed25519`.
- baseline signature profile id: `baseline-v1`.
- baseline signatures include explicit metadata prefix: `sig:ed25519:baseline-v1:<sender>:<nonce>:<state_hash>:<payload_len>`.
- migration fixture matrix helper:
  - `signature_profile_compatibility_fixtures_for_fields(...)`
  - baseline fixture (`baseline-v1`) is accepted.
  - legacy unversioned fixture (`legacy-unversioned`) is rejected.
  - unknown profile fixture (`baseline-v0`) is rejected.
  - unknown algorithm fixture (`secp256k1+baseline-v1`) is rejected.
- signature-profile drift between transaction and signer paths is rejected (`Regression: #400`).
- non-versioned signature profile is rejected (`Regression: #404`).
- algorithm/profile metadata drift or downgrade is rejected (`Regression: #677`).
- compatibility fixture matrix remains aligned between signer and transaction verification (`Regression: #677`).

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
