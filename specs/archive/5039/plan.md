# Issue #5039 Plan

- Issue: #5039
- Status: Implemented

## Approach
1. Add RED conformance tests (`spec_c01`..`spec_c05`) for:
   - archived and reattached recovery readiness (`Ready`),
   - active partition readiness block (`Blocked`),
   - deterministic historical recovery listing,
   - unknown partition fail-closed lookup.
2. Extend `data_layer_m10_partition_archival` with recoverability projections:
   - readiness decision enum + report struct + reason markers,
   - per-partition readiness evaluator,
   - deterministic historical readiness listing.
3. Re-export new APIs in `crates/kamn-core/src/lib.rs`.
4. Run format/lint/targeted/full regression and shell guardrail evidence.

## Affected Modules
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_recoverability.rs` (new)
- `specs/5039/spec.md`
- `specs/5039/plan.md`
- `specs/5039/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Preserve existing M10 lifecycle behavior while adding read-only readiness
    projections.
  - Keep recoverability report ordering deterministic.
  - Keep implementation Rust-only to avoid shell-surface growth.

## Interface Contract
- Additive public API under existing `kamn_core` M10 exports.
- No new dependencies.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.

## Verification Summary
- RED: `cargo test -p kamn-core --test data_layer_m10_partition_recoverability` (failed before implementation with unresolved recoverability symbols/methods).
- GREEN: `cargo test -p kamn-core --test data_layer_m10_partition_recoverability` (5 passed, 0 failed).
- Regression: `cargo test -p kamn-core` (pass), `cargo clippy -p kamn-core -- -D warnings` (pass), `cargo fmt --check` (pass).
