# Escrow Lifecycle State Machine (Issues #138, #139)

This document captures the first implementation slice for PRD 9.2 escrow lifecycle states.
For PaymentOffer and PaymentConfirm workflow integration controls, see `docs/foundation/task-payment-workflow.md`.

## Scope Delivered
- Added `crates/kamn-core/src/escrow.rs` with:
  - `EscrowStatus` state enum (`Funded`, `PartiallyReleased`, `Released`, `Refunded`, `Disputed`, `Resolved`).
  - `EscrowLifecycle` deterministic transition engine.
  - `EscrowLifecycleError` typed invalid-transition and amount-validation errors.
- Added integration tests in `crates/kamn-core/tests/escrow_lifecycle.rs` for legal and illegal transitions.

## Transition Rules Implemented
- `Funded -> PartiallyReleased` via partial release.
- `Funded/PartiallyReleased -> Released` when remaining amount reaches zero.
- `Funded/PartiallyReleased/Disputed -> Refunded` via remaining refund.
- `Funded/PartiallyReleased -> Disputed`.
- `Disputed -> Resolved` only when release/refund split exactly equals remaining amount.

## Deterministic Safety Checks
- Reject zero-value operations where amount must be positive.
- Reject releases larger than remaining escrow amount.
- Reject invalid transitions from terminal states.
- Reject dispute resolution splits that do not reconcile exactly to remaining funds.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test escrow_lifecycle
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
