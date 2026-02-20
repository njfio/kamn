# Issue #5340 Spec

- Title: Task: harden live-postgres daemon validation slice with gate/deferred conformance
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5338` established the first env-gated PostgreSQL live-integration + daemon runtime validation slice. That slice does not yet enforce deterministic gate-reason semantics when PostgreSQL env vars are unset, and does not provide a dedicated live-gated deferred-path coverage contract in the same lane.

## Acceptance Criteria
- AC-1: daemon test utilities expose deterministic env-gate decision markers for PostgreSQL URL resolution (`live_postgres_env_unset` and `live_postgres_adapter_connected`).
- AC-2: add live-gated daemon validation coverage for the deferred Phase-6 path when shutdown signals are configured.
- AC-3: `docs/ops/configuration.md` documents explicit marker contracts and commands for env-gate reason and deferred live slice validation.
- AC-4: docs-contract tests fail closed when the new marker contracts drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only gate/deferred slice hardening in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- Ops marker contract additions in `docs/ops/configuration.md`.
- Docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- R45 review narrative refinement for this follow-up step.

Out of scope:
- Production runtime logic changes.
- Multi-node/load/perf expansion.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | env vars unset under lock | deterministic marker `live_postgres_env_unset` with no resolved URL |
| C-02 | AC-1 | Unit | both env vars set | `KAMN_TEST_POSTGRES_URL` takes precedence and marker is connected |
| C-03 | AC-2 | Integration | live URL configured + daemon shutdown signal | deferred Phase-6 reason projected with live adapter connect/migrate preflight |
| C-04 | AC-3/AC-4 | Functional/Conformance | docs marker section assertions | gate/deferred markers + command references present and enforced |
| C-05 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::regression_runtime_daemon_live_postgres_validation_slice_reports_unset_env_gate_reason -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::unit_runtime_daemon_live_postgres_validation_slice_prefers_kamn_test_postgres_url -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_deferred_path -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_gate_and_deferred_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Env-gate reason codes become deterministically asserted in tests.
- Deferred live-postgres daemon runtime slice is covered and documented.
- Ops/docs marker drift for this slice is prevented by docs-contract tests.
