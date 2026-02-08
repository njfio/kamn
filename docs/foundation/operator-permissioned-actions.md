# Permissioned Operator Actions via Operator-Binding Rules (Issues #198 / #199)

This document captures the first implementation slice for permissioned operator control actions with explicit binding-based authorization.

## Scope Delivered
- Added `crates/kamn-core/src/operator_actions.rs` with:
  - `PermissionedOperatorActionService` for configure, revoke-binding, and read-history operations.
  - `OperatorActionAuditRecord` and `OperatorActionOutcome`.
  - deterministic per-agent configuration state mutation.
  - typed errors via `OperatorActionServiceError`.
- Added integration and regression tests in `crates/kamn-core/tests/operator_permissioned_actions.rs`.

## Authorization Rules
- `configure(...)` requires binding permission `OperatorBindingAction::Configure`.
- `revoke_binding(...)` requires revoke capability enforced by `OperatorBindingEngine`.
- `read_history(...)` requires binding permission `OperatorBindingAction::ReadHistory`.
- Unauthorized requests return explicit binding errors and append denied audit entries.

## Audit and Safety Guarantees
- Every action attempt emits an audit record with:
  - agent DID, operator DID, action, target, timestamp, and outcome.
- Unauthorized configure attempts do not mutate configuration state.
- Revoked bindings cannot be reused for follow-up configuration actions.

## Fast and Cost-Effective Validation
Run targeted checks first:

```bash
cargo test -p kamn-core --test operator_permissioned_actions --test operator_permissioned_actions_docs
cargo fmt --check
cargo clippy -p kamn-core -- -D warnings
```

Then run crate regression:

```bash
cargo test -p kamn-core
```
