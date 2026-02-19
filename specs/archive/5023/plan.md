# Issue #5023 Plan

- Issue: #5023
- Status: Implemented

## Approach
1. Add red conformance tests for C-01..C-06 in `kamn-core`:
   - telemetry ingestion validation and bucket indexing,
   - deterministic hourly/daily aggregate and network summary outputs,
   - owner billing daily projection calculations,
   - owner-scope fail-closed query boundaries.
2. Implement `data_layer_m7_timeseries_telemetry` module with:
   - time-series point registry,
   - aggregate rollup builders,
   - owner billing projection service.
3. Re-export M7 contracts from `crates/kamn-core/src/lib.rs`.
4. Execute format/lint/scoped/full regression and finalize lifecycle markers.

## Affected Modules
- `crates/kamn-core/src/data_layer_m7_timeseries_telemetry.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + re-exports)
- `crates/kamn-core/tests/data_layer_m7_timeseries_telemetry.rs` (new)
- `specs/5023/spec.md`
- `specs/5023/plan.md`
- `specs/5023/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Use explicit bucket derivation helpers for deterministic rollup grouping.
  - Validate requester-owner scope at query entry points.
  - Enforce stable sort and tie-break semantics for all aggregate projections.
  - Keep implementation Rust-only to preserve shell ratio constraints.

## Interface Contract
- Additive public API under `kamn_core::data_layer_m7_timeseries_telemetry::*`.
- No dependency additions.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.

## Outcome
- Added the M7 telemetry/billing contract module and public exports in `kamn-core`.
- Landed conformance coverage for C-01..C-05 and validated full crate regression.
- Kept shell/workflow/python/template surface unchanged.
