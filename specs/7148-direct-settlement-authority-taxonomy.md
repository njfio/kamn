# Issue #7148: Align Direct-Settlement Rejection With Authority Ordering

## Objective

Align the direct-settlement override regression with the independent
service-authority verification order established by #7135.

## Inputs And Outputs

- Input: a required-devnet demo configuration plus an independently generated actor
  receipt bundle whose authority facts do not match the demo transcript.
- Output: rejection with `PI_SERVICE_AUTHORITY_MISMATCH`.

## Boundaries And Non-Goals

- Do not change production verifier ordering or error translation.
- Do not accept command-override settlement evidence.
- Do not reclassify settlement evidence drift that passes actor authority checks.
- Do not change durable receipt-chain, Pi transport, or governance behavior.

## Failure Modes

- The regression expects settlement validation even though actor authority fails first.
- Genuine settlement evidence drift is incorrectly reclassified as actor authority drift.
- Override-only required-devnet evidence is accepted.

## Acceptance Criteria

- [x] The direct-settlement override regression expects
  `PI_SERVICE_AUTHORITY_MISMATCH`.
- [x] Genuine settlement evidence drift retains `SETTLEMENT_EVIDENCE_INVALID`.
- [x] The focused direct-settlement and settlement-evidence targets pass.
- [x] Formatting and strict Clippy pass.

## Files To Touch

- `specs/7148-direct-settlement-authority-taxonomy.md`
- `crates/kamn-e2e-harness/tests/mvp_demo_direct_settlement_contract.rs`

## Error Semantics

- Actor service-authority or projection disagreement:
  `PI_SERVICE_AUTHORITY_MISMATCH`.
- Settlement, RPC, or duplicate-action disagreement reached after authority verification:
  `AGENT_TRANSACTION_SETTLEMENT_INVALID` or `SETTLEMENT_EVIDENCE_INVALID`.

## Test Plan

### RED

- Name the stale direct-settlement expectation and reproduce its mismatch.

### GREEN

- Align the expectation with the first failing service-authority boundary.

### REFACTOR

- Name the expected category for the semantic boundary.

### INTEGRATION

- Run the direct-settlement target, focused genuine-settlement drift tests, formatting,
  and strict Clippy.

## Verification Evidence

- `cargo fmt --all -- --check`
- `cargo test -p kamn-e2e-harness --test mvp_demo_direct_settlement_contract
  --test independent_settlement_evidence_verifier_contract
  --test independent_agent_transaction_verifier_contract`
  - Result: 17 passed, 0 failed.
- `cargo clippy -p kamn-e2e-harness --test mvp_demo_direct_settlement_contract
  --test independent_settlement_evidence_verifier_contract
  --test independent_agent_transaction_verifier_contract -- -D warnings`
