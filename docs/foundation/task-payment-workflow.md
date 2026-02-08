# Task Payment Offer/Confirm Workflow (Issue #216)

This document captures the first implementation slice for integrating `PaymentOffer` and `PaymentConfirm` with task completion and escrow release flow.

## Scope Delivered
- Added `crates/kamn-core/src/task_payment.rs` with:
  - `PaymentOffer` and `PaymentConfirm` message models.
  - `TaskPaymentWorkflow` for offer registration and confirm-triggered escrow release.
  - `TaskPaymentError` typed failures for task-state, escrow, and authorization mismatches.
- Added integration tests in `crates/kamn-core/tests/task_payment_workflow.rs`.

## Deterministic Validation Rules
- Payment offers require:
  - non-empty task and escrow references.
  - positive payment amount.
  - valid payer/payee DIDs.
  - target task in `Completed` state.
  - offered amount less than or equal to escrow remaining balance.
- Payment confirms require:
  - matching task and escrow references for an existing offer.
  - confirmer DID equal to offer payer DID.
  - single-use confirmation (duplicates are rejected).

## Escrow Release Behavior
- Successful `PaymentConfirm` calls `EscrowLifecycle::release(amount)` using the previously accepted offer amount.
- Escrow transition and amount validation errors are surfaced as typed workflow errors.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test task_payment_workflow
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
