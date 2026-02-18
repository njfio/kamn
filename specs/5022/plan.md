# Issue #5022 Plan

- Issue: #5022
- Status: Implemented

## Approach
1. Add red conformance tests for C-01..C-06 in `kamn-core`:
   - graph node/edge registration contracts,
   - cross-owner fail-closed validations,
   - trust propagation ranking determinism with bounded depth,
   - portability edge projection ordering and field completeness.
2. Implement `data_layer_m6_graph_integration` module with:
   - owner-scoped graph registry,
   - trust propagation service over relationship edges,
   - portability export projection API.
3. Re-export M6 contracts from `crates/kamn-core/src/lib.rs`.
4. Execute format/lint/scoped/full regression and finalize lifecycle markers.

## Affected Modules
- `crates/kamn-core/src/data_layer_m6_graph_integration.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + re-exports)
- `crates/kamn-core/tests/data_layer_m6_graph_integration.rs` (new)
- `specs/5022/spec.md`
- `specs/5022/plan.md`
- `specs/5022/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Enforce deterministic sort/tie-break logic for propagation outputs.
  - Validate owner-scope boundaries at registration and query entry points.
  - Keep graph portability output schema explicit to prevent adapter drift.
  - Keep implementation Rust-only to preserve shell ratio constraints.

## Interface Contract
- Additive public API under `kamn_core::data_layer_m6_graph_integration::*`.
- No dependency additions.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.

## Execution Outcome
- Added `data_layer_m6_graph_integration` module implementing owner-scoped graph
  node/edge registration, trust propagation scoring, and portability edge projection contracts.
- Added and passed conformance tests `spec_c01`..`spec_c05` in `kamn-core`.
- Kept implementation Rust-only; no shell/python/workflow/template changes.
