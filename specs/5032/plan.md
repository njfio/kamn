# Issue #5032 Plan

- Issue: #5032
- Status: Implemented

## Approach
1. Add RED tests for determinism stable/drifted/error paths and reason-constant
   assertions.
2. Implement additive M3 determinism contracts:
   `DataLayerM3BlindIndexDeterminismInput`,
   `DataLayerM3BlindIndexDeterminismDecision`,
   `DataLayerM3BlindIndexDeterminismReport`, and
   `DataLayerM3SearchCatalog::evaluate_blind_index_determinism(...)`.
3. Export determinism reason-marker constants and keep existing search behavior
   unchanged.
4. Run scoped/full regression gates and shell guardrail evidence commands.

## Affected Modules
- `crates/kamn-core/src/data_layer_m3_blind_index_search.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m3_blind_index_search.rs`
- `specs/5032/spec.md`
- `specs/5032/plan.md`
- `specs/5032/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep API additive and preserve existing M3 query behavior and ordering.
  - Keep determinism evidence fields deterministic and baseline-ordered.
  - Keep work Rust-only to guarantee `shell_loc_delta_actual = 0`.

## Interface Contract
- Additive API and exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped additive contract.
