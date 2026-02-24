# Spec: Issue #5854 - Close #5853 Residual Run-Path Mutation Escapes

- Issue: #5854
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
`cargo mutants` reported surviving run-path mutants in `crates/kamn-node/src/main.rs` after #5853. Survivors are concentrated in `run()` endpoint runtime-mode guards plus a top-level `run() -> Ok(())` function-value replacement, indicating insufficient direct executable coverage for fail-closed entrypoint behavior and runtime-mode guard contracts.

## Scope
In scope:
- Add deterministic tests that kill the known run-path mutation survivors in `main.rs`.
- Refactor run-path guard logic into focused helpers where needed to make behavior directly testable.
- Add entrypoint-level fail-closed coverage for invalid runtime-mode inputs.
- Re-run in-diff mutation testing for #5854 changes and close unexplained misses.

Out of scope:
- New features beyond run-path mutation hardening.
- Protocol/schema/wire-format changes.
- Cross-crate architectural changes.

## Acceptance Criteria
- AC-1: Service API runtime-mode guard logic is directly tested and fails closed for non-`api`/non-`full` modes.
- AC-2: Observability runtime-mode full-supervisor skip guard logic is directly tested and mutation-resistant.
- AC-3: Entrypoint execution fails closed for invalid runtime-mode CLI input, catching `run() -> Ok(())` mutant behavior.
- AC-4: `cargo mutants --in-diff` for #5854 has no unexplained missed mutants in touched run-path surfaces.
- AC-5: Targeted unit/regression/integration tests pass for touched run-path behavior.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Regression | runtime mode classifier input `bootstrap`/`daemon` | deterministic `ConfigError::RuntimeDaemonLifecycle` containing `service api endpoint requires runtime-mode api or full` |
| C-02 | AC-1 | Unit/Regression | runtime mode classifier input `api` and `full` | `api` => in-process serve mode, `full` => supervisor-skip mode |
| C-03 | AC-2 | Unit/Regression | observability guard input `full` vs non-`full` | `full` => skip in-process endpoint path; non-`full` => do not skip |
| C-04 | AC-3 | Integration/Regression | binary run `kamn-node --runtime-mode invalid-mode` | non-zero exit and stderr contains invalid runtime-mode error signal |
| C-05 | AC-4 | Mutation | `cargo mutants --in-diff /tmp/issue5854.diff -p kamn-node` | no unexplained misses in touched run-path surfaces |

## Test Mapping
- `cargo test -p kamn-node regression_run_path_service_api_runtime_mode_classifier_rejects_non_api_non_full_modes`
- `cargo test -p kamn-node regression_run_path_service_api_runtime_mode_classifier_routes_api_and_full_modes_deterministically`
- `cargo test -p kamn-node regression_run_path_observability_full_supervisor_skip_gate_is_explicit`
- `cargo test -p kamn-node --test runtime_entrypoint_invalid_mode`
- `cargo mutants --in-diff /tmp/issue5854.diff -p kamn-node`

## Success Metrics / Observable Signals
- Prior #5853 run-path survivors are caught by deterministic tests or eliminated via testable guard seams.
- In-diff mutation run for #5854 reports no unexplained misses for touched run-path logic.
- Entrypoint invalid runtime-mode behavior is fail-closed and executable in CI/test harness.
