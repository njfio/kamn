# Issue #7154: Use Bound Runtime Receipt Fixture

## Objective

Align the runtime receipt-chain success contract with transaction-bound v2 service
authority while preserving its negative and privacy contracts.

## Inputs And Outputs

- Input: transaction-bound v2 actor receipts.
- Output: a service-authority summary with canonical actions, receipt-chain commitment,
  and public commitment.

## Boundaries And Non-Goals

- Do not change production receipt-chain construction or verifier behavior.
- Do not accept generic unbound v2 authority.
- Do not expose transport response digests or participant roles.
- Do not change settlement, Pi transport, or governance behavior.

## Failure Modes

- The success case uses generic v2 receipts and fails authority verification.
- Bound authority changes the canonical action inventory or commitments.
- Private transport or role fields enter the public summary.
- Negative missing, duplicate, failed, reordered, privacy, or fact-drift cases pass.

## Acceptance Criteria

- [ ] The success case uses transaction-bound v2 actor receipts.
- [ ] The summary retains canonical actions and commitment fields.
- [ ] Private transport and participant-role fields remain absent.
- [ ] All negative authority cases remain rejected.
- [ ] The complete four-case target passes.
- [ ] Formatting and strict Clippy pass.

## Files To Touch

- `specs/7154-bound-runtime-receipt-fixture.md`
- `crates/kamn-e2e-harness/tests/mvp_demo_runtime_receipt_chain_contract.rs`

## Error Semantics

- Bound v2 authority verifies successfully.
- Unbound, incomplete, duplicate, failed, reordered, private, or fact-drift authority
  returns the owning `PI_*` authority category.

## Test Plan

### RED

- Isolate the stale success fixture and reproduce `PI_SERVICE_AUTHORITY_MISMATCH`.

### GREEN

- Use the transaction-bound v2 fixture for the success case.

### REFACTOR

- Name the bound success-fixture setup explicitly.

### INTEGRATION

- Run the complete target, formatting, strict Clippy, and adjacent service-authority
  contracts.
