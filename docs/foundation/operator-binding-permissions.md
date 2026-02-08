# Operator Binding Proofs and Permission Enforcement (Issues #230, #231)

This document captures the first implementation slice for optional operator binding proofs and permission gates for operator control actions.

## Scope Delivered
- Added `crates/kamn-core/src/operator_binding.rs` with:
  - `OperatorBindingEngine` for binding registration, authorization, and revocation.
  - `OperatorBindingAction` permission gates: `Configure`, `Revoke`, and `ReadHistory`.
  - `OperatorBindingProof` model and deterministic proof validation checks.
  - Typed errors via `OperatorBindingError` for missing binding, revoked binding, unauthorized actions, and proof validation failures.
- Added integration tests in `crates/kamn-core/tests/operator_binding_permissions.rs`.

## Validation Rules
- Agent DID must parse as `kamn:did:agent:<id>`.
- Operator DID must match `kamn:did:human:<id>` with lowercase alphanumeric, `_`, or `-`.
- Permission sets must be non-empty at registration.
- Proof is optional, but when present:
  - `type_name` must be `Ed25519Signature2020`.
  - `created`, `verification_method`, and `proof_value` must be non-empty.
  - `verification_method` must start with `<operator_did>#`.

## Permission Enforcement
- `authorize(...)` enforces action-level permission membership per binding.
- `revoke_binding(...)` requires `Revoke` permission and marks binding as revoked.
- Any action against a revoked binding fails with a typed `RevokedBinding` error.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test operator_binding_permissions
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
