# Issue #5009 Plan

- Issue: #5009
- Status: Implemented

## Approach
1. Deliver M6 graph contracts through child task `#5022`:
   - owner-scoped graph node/edge registration,
   - deterministic bounded-depth trust-propagation ranking,
   - deterministic portability/export projections with fail-closed access controls.
2. Preserve additive exports in `kamn-core` for downstream integration lanes.
3. Validate with scoped suite `data_layer_m6_graph_integration` and crate-level regression.
4. Keep delivery Rust-only for this story to preserve shell budget neutrality.

## Affected Modules
- `crates/kamn-core/src/data_layer_m6_graph_integration.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m6_graph_integration.rs`
- `specs/5009/spec.md`
- `specs/5009/plan.md`
- `specs/5009/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep trust-propagation ordering deterministic in conformance coverage.
  - Preserve strict owner scoping across registration and query surfaces.
  - Preserve rust-only implementation to avoid shell-surface growth.

## Interface Contract
- Additive API/exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this bounded additive story closure.
