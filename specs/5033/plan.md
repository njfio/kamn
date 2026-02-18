# Issue #5033 Plan

- Issue: #5033
- Status: Implemented

## Approach
1. Add RED tests for reconciliation match/mismatch/non-terminal error paths and
   replace transition/visibility reason-string assertions with constants.
2. Implement additive reconciliation contracts in M4:
   `DataLayerM4SettlementEvidenceReconciliationDecision`,
   `DataLayerM4SettlementEvidenceReconciliationReport`, and
   `DataLayerM4SettlementEvidenceRegistry::reconcile_against_escrow(...)`.
3. Export and use stable reason-marker constants for transition and visibility
   outcomes in module and tests.
4. Run scoped/full regression gates plus shell guardrail evidence commands.

## Affected Modules
- `crates/kamn-core/src/data_layer_m4_escrow_integration.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m4_escrow_integration.rs`
- `specs/5033/spec.md`
- `specs/5033/plan.md`
- `specs/5033/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep API additive and preserve existing M4 transition/visibility behavior.
  - Make reconciliation deterministic on latest evidence record only.
  - Keep work Rust-only to guarantee `shell_loc_delta_actual = 0`.

## Interface Contract
- Additive exports in `kamn-core` only.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped additive contract.
