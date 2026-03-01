# Spec: Issue #6293 - Bridges settlement decision API + integration lane

## Objective

Add a canonical settlement decision API in `kamn-bridges` that maps normalized cross-chain
receipt finality into deterministic bridge actions (`settle`, `defer`, `reject`) with typed
reasons, and establish first crate-level integration coverage.

## Inputs/Outputs

- Inputs:
  - `CrossChainReceiptProof`
- Outputs:
  - `CrossChainSettlementDecision`:
    - `Settle`
    - `DeferPendingFinality`
    - `Reject(CrossChainSettlementRejectionReason)`

## Boundaries/Non-goals

- In scope:
  - New settlement policy module/API in `kamn-bridges`.
  - Integration tests in `crates/kamn-bridges/tests/`.
- Out of scope:
  - Modifying normalization logic semantics in `cross_chain_receipt`.
  - New networks/adapters.

## Failure Modes

- FM-1: pending receipts incorrectly settle.
- FM-2: failed receipts incorrectly defer.
- FM-3: invalid proofs do not map to typed invalid-proof rejection.
- FM-4: integration lane missing or not exercising public API.

## Acceptance Criteria

- AC-1: public settlement decision API exists and consumes `CrossChainReceiptProof`.
- AC-2: `Final -> Settle`, `Pending -> DeferPendingFinality`, `Failed -> Reject(FailedReceipt)`.
- AC-3: normalization failure maps to `Reject(InvalidProof(...))`.
- AC-4: `crates/kamn-bridges/tests/bridge_settlement_integration.rs` covers settle/defer/reject
  and invalid-proof rejection.
- AC-5: existing `kamn-bridges` tests remain green.

## Files To Touch

- `crates/kamn-bridges/src/lib.rs`
- `crates/kamn-bridges/src/cross_chain_settlement.rs`
- `crates/kamn-bridges/tests/bridge_settlement_integration.rs`
- `specs/6293-bridges-settlement-decision-integration.md`

## Error Semantics

- Settlement API itself is infallible and always returns a decision.
- Invalid proof inputs return deterministic typed rejection reason:
  `CrossChainSettlementRejectionReason::InvalidProof(CrossChainReceiptNormalizationError)`.

## Test Plan

- RED:
  - Add integration tests calling new settlement API + typed decision/rejection enums.
  - Confirm fail before implementation.
- GREEN:
  - Implement minimal settlement policy mapping over normalization result/finality.
- REFACTOR:
  - Keep mapping logic small and explicit in one helper.
- Verification:
  - `cargo fmt --all --check`
  - `cargo clippy -p kamn-bridges --tests -- -D warnings`
  - `cargo test -p kamn-bridges`
