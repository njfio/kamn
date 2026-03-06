## Objective

Add explicit edge-case coverage for `kamn-live-probe-matrix` so deterministic fail-closed
aggregation and lookup behavior are specified by tests instead of inferred from implementation.

## Inputs/Outputs

- Inputs:
  - `LiveProbeMatrixEntry::new(mode, scenario_id, status)`
  - `LiveProbeMatrixReport::new(entries)`
  - `LiveProbeMatrixReport::status_for(mode, scenario_id)`
  - `LiveProbeMatrixReport::mode_status(mode)`
  - `LiveProbeMatrixReport::overall_status()`
  - `LiveProbeMatrixReport::mode_status_map()`
  - `project_live_probe_matrix_summary(report)`
- Outputs:
  - deterministic aggregate status results for empty, all-skip, and mixed-mode reports
  - deterministic `status_for` lookup behavior for trimmed and empty scenario identifiers

## Boundaries/Non-goals

- No production API changes
- No behavior changes unless a newly added test proves current behavior violates the issue
- No CI, workflow, or dependency changes
- No new runtime wiring; this is crate-level integration coverage only

## Failure modes

- Empty reports accidentally aggregate to a concrete status instead of `None`
- All-`SKIP` reports regress to fail-closed `FAIL` instead of deterministic `SKIP`
- `status_for` stops trimming lookup identifiers or starts accepting empty lookups as valid
- `mode_status_map` includes absent modes or returns non-deterministic aggregate results

## Acceptance criteria

- [x] A test proves empty reports return `None` for `overall_status()`
- [x] A test proves empty reports return an empty map from `mode_status_map()`
- [x] A test proves all-`SKIP` entries aggregate to `Some(SKIP)` for both `mode_status()` and `overall_status()`
- [x] A test proves summary projection preserves all-`SKIP` counts and overall `Some(SKIP)`
- [x] A test proves `status_for()` trims surrounding whitespace in lookup identifiers
- [x] A test proves `status_for()` returns `None` for empty lookup identifiers
- [x] A test proves `mode_status_map()` omits absent modes and returns deterministic per-mode aggregates for mixed modes
- [x] `cargo test -p kamn-live-probe-matrix -- --nocapture` passes

## Files to touch

- `specs/6481-add-live-probe-matrix-edge-case-coverage.md`
- `crates/kamn-live-probe-matrix/tests/live_probe_matrix_edge_cases_contract.rs`
- `crates/kamn-live-probe-matrix/tests/live_probe_matrix_edge_cases.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` (only if test inventory changes)

## Error semantics

- `LiveProbeMatrixEntry::new(...)` continues to reject empty scenario identifiers with
  `LiveProbeMatrixError::EmptyScenarioId`
- `LiveProbeMatrixReport::new(...)` continues to reject duplicate mode/scenario pairs with
  `LiveProbeMatrixError::DuplicateModeScenario`
- Lookup helpers continue to fail closed by returning `None` for invalid empty lookup identifiers
- No new error variants are introduced

## Test plan

- Add a contract test that requires dedicated edge-case test markers in a new integration test file
- Add integration tests for empty report aggregation, all-`SKIP` aggregation, trimmed lookup ids,
  empty lookup ids, and deterministic mixed-mode `mode_status_map()` output
- Run:
  - `cargo test -p kamn-live-probe-matrix --test live_probe_matrix_edge_cases_contract -- --nocapture`
  - `cargo test -p kamn-live-probe-matrix --test live_probe_matrix_edge_cases -- --nocapture`
  - `cargo test -p kamn-live-probe-matrix -- --nocapture`
  - `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` if a new test file changes inventory

## Phase 6 integration evidence

- The real crate path is exercised via `cargo test -p kamn-live-probe-matrix -- --nocapture`
- New integration coverage landed in `crates/kamn-live-probe-matrix/tests/live_probe_matrix_edge_cases.rs`
- Contract coverage landed in `crates/kamn-live-probe-matrix/tests/live_probe_matrix_edge_cases_contract.rs`
- The test inventory baseline was refreshed to account for the two new test targets

## Deviations

- None
