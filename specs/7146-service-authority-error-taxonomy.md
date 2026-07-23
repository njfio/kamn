# Issue #7146: Align Legacy Transcript Rejection With Service-Authority Taxonomy

## Objective

Align the legacy transcript/runtime actor regression with the independent
service-authority error semantics introduced by #7135.

## Inputs And Outputs

- Input: a generated legacy transcript and independent runtime actor artifacts.
- Output: rejection with `PI_SERVICE_AUTHORITY_MISMATCH`.

## Boundaries And Non-Goals

- Do not change verifier execution order or production error translation.
- Do not accept legacy or client-local authority.
- Do not change durable receipt-chain verification.
- Do not change governance-ratio policy or submit a live settlement.

## Failure Modes

- A stale test expects the downstream receipt-chain category even though independent
  actor authority fails first.
- Receipt-chain corruption is incorrectly reclassified as actor authority drift.
- Legacy evidence is accepted.

## Acceptance Criteria

- [x] Legacy transcript/runtime actor disagreement expects
  `PI_SERVICE_AUTHORITY_MISMATCH`.
- [x] Receipt-chain rebuild corruption retains `RECEIPT_CHAIN_INVALID`.
- [x] The complete `mvp_demo_command_contract` target passes.
- [x] Formatting and strict Clippy pass.

## Files To Touch

- `specs/7146-service-authority-error-taxonomy.md`
- `crates/kamn-e2e-harness/tests/mvp_demo_command_contract.rs`

## Error Semantics

- Actor service-authority or projection disagreement:
  `PI_SERVICE_AUTHORITY_MISMATCH`.
- Durable chain parsing, ordering, digest, or commitment disagreement:
  `SERVICE_RECEIPT_CHAIN_INVALID` or its verifier boundary projection.

## Test Plan

### RED

- Reproduce the current assertion mismatch on `main`.

### GREEN

- Align the stale expectation with the documented service-authority category.

### REFACTOR

- Name the expected category locally so the boundary is explicit.

### INTEGRATION

- Run the full command-contract target, formatting, and strict Clippy.

## Verification Evidence

- `cargo fmt --all -- --check`
- `cargo test -p kamn-e2e-harness --test mvp_demo_command_contract`
  - Result: 12 passed, 0 failed.
- `cargo clippy -p kamn-e2e-harness --test mvp_demo_command_contract -- -D warnings`
