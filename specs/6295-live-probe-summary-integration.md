# Spec: Issue #6295 - Live-probe summary projection + integration lane

## Objective

Add a canonical summary projection API for `LiveProbeMatrixReport` and establish first
crate-level integration coverage for `kamn-live-probe-matrix`.

## Inputs/Outputs

- Inputs:
  - `&LiveProbeMatrixReport`
- Outputs:
  - `LiveProbeMatrixSummary` including:
    - `total_entries`
    - `pass_entries`
    - `fail_entries`
    - `skip_entries`
    - `overall_status`

## Boundaries/Non-goals

- In scope:
  - New summary projection module/API.
  - Integration tests under `crates/kamn-live-probe-matrix/tests/`.
- Out of scope:
  - Changing existing aggregation semantics.
  - New probe modes.

## Failure Modes

- FM-1: summary counts do not match report entries.
- FM-2: mixed PASS/SKIP projection does not preserve fail-closed overall status.
- FM-3: integration lane missing for crate public API.

## Acceptance Criteria

- AC-1: public `project_live_probe_matrix_summary(&LiveProbeMatrixReport)` exists.
- AC-2: summary counts match exact entry distribution.
- AC-3: summary `overall_status` matches `LiveProbeMatrixReport::overall_status()`.
- AC-4: integration tests cover all-pass and mixed pass/skip scenarios.
- AC-5: existing `kamn-live-probe-matrix` tests remain green.

## Files To Touch

- `crates/kamn-live-probe-matrix/src/lib.rs`
- `crates/kamn-live-probe-matrix/src/summary_projection.rs`
- `crates/kamn-live-probe-matrix/tests/summary_projection_integration.rs`
- `specs/6295-live-probe-summary-integration.md`

## Error Semantics

- Projection is infallible for validated reports.
- No fallback behavior; summary is deterministic function of report entries.

## Test Plan

- RED:
  - Add integration tests for summary projection API and expected counts/status.
  - Confirm fail before implementation.
- GREEN:
  - Implement minimal summary projection type + function.
- REFACTOR:
  - Keep summary counting helper small and explicit.
- Verification:
  - `cargo fmt --all --check`
  - `cargo clippy -p kamn-live-probe-matrix --tests -- -D warnings`
  - `cargo test -p kamn-live-probe-matrix`
