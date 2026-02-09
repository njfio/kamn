# Task Payment Offer/Confirm Workflow (Issues #216 / #542 / #558)

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
  - `payer_did` equal to task requester DID.
  - `payee_did` equal to task assignee DID.
  - offered amount less than or equal to escrow remaining balance.
- Payment confirms require:
  - matching task and escrow references for an existing offer.
  - confirmer DID equal to offer payer DID.
  - single-use confirmation (duplicates are rejected).
- Timeout refunds require:
  - `current_unix >= timeout_unix` before invoking timeout-conditioned escrow refund.
  - premature timeout refund attempts are rejected (`Regression: #542`).

## Escrow Release Behavior
- Successful `PaymentConfirm` calls `EscrowLifecycle::release(amount)` using the previously accepted offer amount.
- Timeout-based recovery calls `EscrowLifecycle::refund_after_timeout(current_unix, timeout_unix)` to return the remaining balance once the deadline is reached.
- Escrow transition and amount validation errors are surfaced as typed workflow errors.

## Settlement Evidence Schema and Policy Checks
Task payment settlement outcomes must be accompanied by deterministic escrow evidence before payout workflows are treated as complete.

- Evidence bundle command:
  - `bash scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh --output-file /tmp/settlement-evidence.json --escrow-id escrow-001 --settlement-outcome RELEASED --receipt-id receipt-001 --receipt-finality FINAL --expected-release-amount 120 --expected-refund-amount 0 --observed-release-amount 120 --observed-refund-amount 0 --ledger-reference-id ledger-entry-001 --timeout-elapsed false --ci-fast-gate PASS`
- Policy checker command:
  - `bash scripts/escrow/check_settlement_reconciliation_evidence_policy.sh --bundle-file /tmp/settlement-evidence.json`
- Required schema and reason-key markers:
  - `kamn.escrow.settlement-reconciliation.v1`
  - `settlement_reconciliation_reason_codes:GO:v1`
  - `settlement_reconciliation_reason_codes:NO-GO:v1`
- Deterministic settlement path reason-code markers:
  - payout path: `settlement_path_payout`
  - refund path: `settlement_path_refund`
  - dispute path: `settlement_path_dispute`
- Fail-closed rule:
  - settlement evidence schema drift must fail closed (`Regression: #906`).

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test task_payment_workflow
bash scripts/escrow/test_generate_settlement_reconciliation_evidence_bundle.sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
