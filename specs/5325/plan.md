# Issue #5325 Plan

## Approach
1. Keep `data_layer_m10_partition_archival.rs` as stable root module path.
2. Extract implementation-heavy concerns into child modules:
   - `error.rs` (error taxonomy + mapping helpers)
   - `shared.rs` (shared validators and deterministic helpers)
   - `retry.rs` (archival retry decision and validation)
   - `phase6.rs` (execution/scheduler/runtime/evidence logic)
   - `registry.rs` (partition lifecycle registry operations)
3. Re-export required public items from root to preserve API shape.
4. Run scoped M10/public API/doc-contract suites and strict clippy.

## Affected Modules
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/error.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/shared.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/retry.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/phase6.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs`
- `specs/5325/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: behavior drift while moving helpers across modules.
  - Mitigation: preserve function bodies and reason-code constants; run existing conformance suite unchanged.
- Risk: accidental API visibility changes.
  - Mitigation: keep root re-exports and run `public_api_surface_policy` tests.

## Interfaces and Contracts
- Module path contract unchanged: `kamn_core::data_layer_m10_partition_archival`.
- Reason-code taxonomy constants unchanged.
- Existing test selectors stay valid.
