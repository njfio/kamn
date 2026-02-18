# Issue #5034 Plan

- Issue: #5034
- Status: Implemented

## Approach
1. Add RED conformance tests for recall-drift stable/degraded/error paths and
   replace anomaly/query reason-string assertions with exported constants.
2. Implement additive M5 recall-drift contracts:
   `DataLayerM5RecallDriftEvaluationInput`,
   `DataLayerM5RecallDriftDecision`,
   `DataLayerM5RecallDriftReport`, and
   `DataLayerM5EmbeddingRegistry::evaluate_recall_drift(...)`.
3. Export stable reason-marker constants for semantic/anomaly/privacy paths and
   replace literal return strings in the M5 implementation.
4. Run scoped and full regression gates plus shell-loc/ratio guardrails.

## Affected Modules
- `crates/kamn-core/src/data_layer_m5_vector_integration.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m5_vector_integration.rs`
- `specs/5034/spec.md`
- `specs/5034/plan.md`
- `specs/5034/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep behavior additive; do not change existing semantic/anomaly outputs.
  - Preserve deterministic ordering in recall-drift evidence fields.
  - Keep implementation Rust-only to guarantee `shell_loc_delta_actual = 0`.

## Interface Contract
- Additive API and type exports within `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped additive contract.
