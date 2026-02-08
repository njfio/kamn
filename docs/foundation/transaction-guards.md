# Transaction Guards (Issue #78)

This document defines the baseline deterministic transaction guards implemented in `kamn-core`.

## Invariants Enforced
- Non-empty transaction envelope fields (`id`, `sender`, `payload`, `state_hash`, `signature`).
- Positive nonce values.
- Per-sender nonce sequencing (first nonce `1`, then increment by `1`).
- State-hash match against the currently expected chain/app state hash.
- Deterministic signature integrity check (baseline placeholder scheme).
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
- signature-profile drift between transaction and signer paths is rejected (`Regression: #400`).

## Validation
Run from repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```
