# Issue #5029 Plan

- Issue: #5029
- Status: Implemented

## Approach
1. Add RED tests for M0 conformance-matrix stable/drift paths and invalid input
   fail-closed behavior.
2. Implement additive M0 contracts:
   `DataLayerM0ConformanceInvariant`,
   `DataLayerM0ConformanceMatrixCase`,
   `DataLayerM0ConformanceMatrixEvidence`,
   `DataLayerM0ConformanceMatrixDecision`,
   `DataLayerM0ConformanceMatrixReport`, and
   `evaluate_data_layer_m0_conformance_matrix(...)`.
3. Export stable decision reason constants and re-export new symbols via
   `kamn-core` crate root.
4. Run scoped/full regression gates plus shell guardrail evidence commands.

## Affected Modules
- `crates/kamn-core/src/data_layer_m0.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m0_contract.rs`
- `specs/5029/spec.md`
- `specs/5029/plan.md`
- `specs/5029/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep API additive and preserve existing M0 behavior.
  - Keep evidence ordering deterministic by preserving input case order.
  - Keep work Rust-only to guarantee `shell_loc_delta_actual = 0`.

## Interface Contract
- Additive API/exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped additive contract.
