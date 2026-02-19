# Issue #5026 Plan

- Issue: #5026
- Status: Implemented

## Approach
1. Add RED conformance tests (`spec_c01`..`spec_c05`) for:
   - deterministic partition naming and future-horizon planning,
   - retention-window archival eligibility with shred-complete requirements,
   - archival-index export metadata and re-attachment transitions,
   - invalid/illegal transition fail-closed behavior.
2. Implement `data_layer_m10_partition_archival` module with:
   - monthly partition registry + naming helpers,
   - archival candidate planner and archival index records,
   - re-attachment lifecycle transition enforcement.
3. Re-export M10 APIs in `crates/kamn-core/src/lib.rs`.
4. Run format/lint/targeted/full regression and finalize lifecycle evidence.

## Affected Modules
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + exports)
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs` (new)
- `specs/5026/spec.md`
- `specs/5026/plan.md`
- `specs/5026/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Use deterministic month arithmetic and canonical partition naming utilities.
  - Enforce lifecycle transition guards (active -> archived -> reattached only).
  - Validate archival eligibility rules fail-closed for incomplete partitions.
  - Keep implementation Rust-only to avoid shell-surface growth.

## Interface Contract
- Additive public API under `kamn_core::data_layer_m10_partition_archival::*`.
- No new dependencies.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.

## Verification Summary
- RED: `cargo test -p kamn-core --test data_layer_m10_partition_archival` (failed before implementation with unresolved `DataLayerM10*` symbols).
- GREEN: `cargo test -p kamn-core --test data_layer_m10_partition_archival` (5 passed, 0 failed).
- Regression: `cargo test -p kamn-core` (pass), `cargo clippy -p kamn-core -- -D warnings` (pass), `cargo fmt --check` (pass).
