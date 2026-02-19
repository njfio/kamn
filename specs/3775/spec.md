# Issue #3775 Spec

- Title: Task: standardize node tracing subscriber and runtime event taxonomy
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
`kamn-node` needed deterministic tracing/logging bootstrap and taxonomy drift protection to make runtime events operationally reliable across bootstrap/full/kolme-live paths.

## Acceptance Criteria
- AC-1: Runtime events are emitted through standardized tracing/logging wiring across runtime modes.
- AC-2: Critical event fields (`execution_id`, `runtime_mode`, `route`, `reason_code`, checkpoint failures) are deterministic and tested.
- AC-3: Legacy logging controls/parity remain explicit (level/format mapping, fail-closed invalid config behavior).
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Scope
In scope:
- Child subtask deliveries:
  - `#3782` tracing subscriber bootstrap wiring across runtime modes
  - `#3783` tracing taxonomy drift/docs parity enforcement
- Contract docs and docs-contract tests:
  - `docs/observability/contracts.md`
  - `crates/kamn-node/tests/observability_contracts_docs.rs`
- Runtime verification in `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `specs/3775/{spec.md,plan.md,tasks.md}`

Out of scope:
- Remote SaaS telemetry backend rollout
- New event families unrelated to runtime operations

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | bootstrap/full runtime mode startup tracing tests | Runtime startup emits deterministic structured markers |
| C-02 | AC-2 | Unit/Integration | tracing taxonomy docs-contract tests | Required vocabulary and runtime source parity remain enforced fail closed |
| C-03 | AC-3 | Unit/Regression | log config parser + invalid config fail-closed test | level/format controls parse deterministically; invalid config fails closed |
| C-04 | AC-4 | Regression | lint + shell guardrails | quality and governance gates green |

## Test Mapping
- `cargo test -p kamn-node --test observability_contracts_docs`
- `cargo test -p kamn-node main_tests::runtime_tests::integration_bootstrap_runtime_emits_structured_marker -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::integration_runtime_full_emits_ordered_bootstrap_readiness_markers -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::unit_log_config_parses_level_and_format_inputs -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::regression_invalid_log_level_config_fails_closed -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3775.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3775.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3775.json`

## Success Metrics
- Parent ACs satisfied by merged child subtasks `#3782` and `#3783`.
- Tracing bootstrap and taxonomy drift contracts remain pinned in docs/tests.
- No shell LOC increase for this closure issue (`shell_loc_delta_actual=0` target).
