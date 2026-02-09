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

## Transition Evidence and Reason-Code Contract (Issue #903)
- `EscrowLifecycle::apply_transition_with_evidence(...)` emits deterministic mutation evidence for authorized transitions:
  - `EscrowTransitionEvidence { from, action, to, reason_code }`
  - allowed transition reason code: `escrow_transition_allowed`
- Rejected transition reason codes are deterministic via `EscrowLifecycleError::reason_code()`:
  - `escrow_amount_zero`
  - `escrow_receipt_missing`
  - `escrow_receipt_finality_invalid`
  - `escrow_amount_invalid`
  - `escrow_transition_invalid`
  - `escrow_resolution_mismatch`
  - `escrow_timeout_not_elapsed`
  - `escrow_amount_overflow`
- Receipt-finality settlement outcomes expose deterministic reason codes:
  - `escrow_settlement_finalized`
  - `escrow_settlement_pending_receipt_finality`
  - `escrow_settlement_rejected_receipt_finality`
- Regression policy:
  - transition reason-code drift and illegal transition acceptance fail closed (`Regression: #903`).

## Settlement Reconciliation Evidence Contract (Issue #687)
Settlement reconciliation evidence is captured as machine-readable JSON so release gates can audit escrow outcomes against receipt finality and timeout rules.

- Evidence bundle generator:
  - `bash scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh --output-file /tmp/settlement-evidence.json --escrow-id escrow-001 --settlement-outcome RELEASED --receipt-id receipt-001 --receipt-finality FINAL --expected-release-amount 120 --expected-refund-amount 0 --observed-release-amount 120 --observed-refund-amount 0 --ledger-reference-id ledger-entry-001 --timeout-elapsed false --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/escrow/check_settlement_reconciliation_evidence_policy.sh --bundle-file /tmp/settlement-evidence.json`
- PR fast contract lane:
  - `bash scripts/escrow/run_settlement_reconciliation_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/escrow/run_settlement_reconciliation_deep_lane.sh --output-json settlement-reconciliation-report.json`
- Regression policy:
  - missing or invalid chain receipt evidence forces `NO-GO` (`Regression: #678`).
  - missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`).

## Chain Receipt Finality Adapter Contract (Issue #718)
Escrow settlement transitions must map chain receipt finality to deterministic typed outcomes.

- Core adapter API:
  - `EscrowReceiptFinality::{Final, Pending, Failed}`
  - `EscrowLifecycle::reconcile_receipt_finality(receipt_id, finality, action)`
  - `EscrowSettlementOutcome::{Settled, Pending, Rejected}`
- Fail-closed policy:
  - unknown finality values are rejected by `EscrowReceiptFinality::parse(...)`.
  - pending/failed finality never mutates escrow state.
  - missing receipt evidence returns an explicit typed error.

## Timeout/Finality Race Matrix Evidence (Issue #719)
Timeout-conditioned settlement flows and delayed receipt finality are validated with deterministic race fixtures.

- Race matrix runner:
  - `python3 scripts/escrow/run_settlement_reconciliation_race_matrix.py --fixture fixtures/escrow_reconciliation/finality_race_cases.json --output-json /tmp/settlement-race-report.json`
- Deep lane entrypoint:
  - `bash scripts/escrow/run_settlement_reconciliation_deep_lane.sh --output-json settlement-reconciliation-report.json`
- Regression policy:
  - timeout-before-finality pending receipts and failed receipts force `NO-GO` (`Regression: #678`).

## Local Validation
Run from repository root:

```bash
bash scripts/escrow/test_generate_settlement_reconciliation_evidence_bundle.sh
bash scripts/escrow/test_run_settlement_reconciliation_contract_lane.sh
bash scripts/escrow/test_run_settlement_reconciliation_race_matrix.sh
cargo test -p kamn-core --test escrow_lifecycle
cargo test -p kamn-core --test task_escrow_transition_contracts
cargo test -p kamn-core --test escrow_lifecycle_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
