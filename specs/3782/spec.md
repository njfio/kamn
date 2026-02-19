# Issue #3782 Spec

- Title: Subtask: wire tracing subscriber bootstrap across node runtime modes
- Status: Implemented
- Type: subtask
- Priority: P0
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
`kamn-node` startup logging and tracing bootstrap behavior must stay deterministic across runtime modes, with fail-closed behavior for invalid log configuration and docs-contract coverage to prevent drift.

## Acceptance Criteria
- AC-1: Tracing/logging startup contract markers are documented for `bootstrap`, `full`, and `kolme-live` runtime modes.
- AC-2: Invalid logging configuration fail-closed markers are documented and covered by docs-contract tests.
- AC-3: Unit, Functional, Integration, and Regression evidence is present and passing for startup logging config mapping and fail-closed behavior.

## Scope
In scope:
- `docs/observability/contracts.md`
- `crates/kamn-node/tests/observability_contracts_docs.rs`
- Existing runtime logging/tracing tests in `crates/kamn-node/src/main_tests/runtime_tests.rs` as verification targets
- `specs/3782/{spec.md,plan.md,tasks.md}`

Out of scope:
- External telemetry backend integration
- New dependencies
- Protocol or wire-format changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `docs/observability/contracts.md` startup logging section | Version marker + runtime mode markers + env control markers are present |
| C-02 | AC-2 | Integration | `crates/kamn-node/tests/observability_contracts_docs.rs` startup logging contract tests | Docs-contract assertions fail closed on marker drift |
| C-03 | AC-3 | Functional | `main_tests::runtime_tests::integration_bootstrap_runtime_emits_structured_marker` and `main_tests::runtime_tests::integration_runtime_full_emits_ordered_bootstrap_readiness_markers` | Runtime mode startup emits deterministic structured bootstrap markers |
| C-04 | AC-3 | Regression | `main_tests::runtime_tests::regression_invalid_log_level_config_fails_closed` | Invalid log level config yields deterministic `ConfigError::InvalidLogConfig` |

## Test Mapping
- `cargo test -p kamn-node --test observability_contracts_docs`
- `cargo test -p kamn-node main_tests::runtime_tests::integration_bootstrap_runtime_emits_structured_marker -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::integration_runtime_full_emits_ordered_bootstrap_readiness_markers -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::regression_invalid_log_level_config_fails_closed -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3782.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3782.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3782.json`

## Success Metrics
- Startup logging contract markers are explicit and fail-closed in docs-contract tests.
- Runtime mode startup/fail-closed behavior tests remain green.
- No shell LOC increase for this issue (`shell_loc_delta_actual=0` target).
