# Spec: #5680 External Runtime Probe Execution in E2E Harness

- Issue: #5680
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`kamn-e2e-harness` currently emits deterministic placeholder runtime marker blocks for `runtime_orchestration` and `runtime_lifecycle_execution` even when `--enable-external-execution` is set. This creates false-positive live-runtime status and does not validate configured binaries beyond preflight checks.

## Scope
### In Scope
- Add executable probe execution for configured external binaries in run mode when `external_execution=true`.
- Map probe outcomes into `runtime_orchestration`, `runtime_lifecycle_execution`, and `runtime_validation_execution` marker blocks.
- Preserve current non-external (`external_execution=false`) behavior.
- Preserve deterministic error handling (no panic paths) for probe failures.

### Out of Scope
- Full process lifecycle orchestration for postgres/kolme/kamn node daemons.
- Scenario semantic changes and new scenario business logic.
- Changes to run/verify CLI flags.

## Acceptance Criteria
### AC-1 Probe-backed external orchestration markers
Given `execute_run_contract` is called with `external_execution=true` and valid executable paths,
When run output is generated,
Then external runtime marker blocks are derived from real probe outcomes rather than static scaffold constants.

### AC-2 Deterministic failure reporting
Given a probe invocation fails (non-zero exit),
When run output is generated,
Then the corresponding runtime marker status is `FAIL` and the run output remains structured JSON without panic.

### AC-3 Backward compatibility for non-external mode
Given `execute_run_contract` is called with `external_execution=false`,
When run output is generated,
Then existing SKIP semantics and marker structure remain unchanged.

### AC-4 Regression safety
Given existing preflight and command contract assertions,
When test suites run,
Then external-execution preflight contracts continue passing with updated runtime marker expectations.

## Conformance Cases
- C-01 (AC-1): external execution with executable binaries returns PASS probe-backed runtime orchestration/lifecycle markers.
- C-02 (AC-2): external execution with a probe script exiting non-zero returns FAIL statuses in runtime markers and validation summary.
- C-03 (AC-3): non-external execution preserves SKIP runtime marker behavior.
- C-04 (AC-4): preflight error contracts for missing/non-file/non-executable paths remain unchanged.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` passes with probe-backed external runtime assertions.
- `cargo test -p kamn-e2e-harness` remains green.
- `cargo fmt --all --check` and `cargo clippy -p kamn-e2e-harness -- -D warnings` pass.
