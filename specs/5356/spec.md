# Issue #5356 Spec

- Title: Task: add asymmetric parallel lane contracts for live-postgres daemon validation
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5354` codified deterministic bounded parallel role-pair lanes for symmetric leg profiles, but mixed-cadence asymmetric lanes are not explicitly contract-enforced. Concurrent lanes with different max-ticks/tick-interval pairs per leg could drift in reason/taxonomy behavior without dedicated coverage.

## Acceptance Criteria
- AC-1: daemon validation tests assert deterministic reason/taxonomy outcomes for canonical asymmetric parallel lanes across applied/deferred scenarios.
- AC-2: canonical asymmetric lane constants and ordering are explicitly asserted in test code.
- AC-3: `docs/ops/configuration.md` includes explicit `#5356` asymmetric lane marker contracts and validation commands.
- AC-4: docs-contract tests fail closed on asymmetric lane marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only asymmetric parallel lane assertions in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- Ops marker additions in `docs/ops/configuration.md`.
- Docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- R45 review narrative refinement for this increment.

Out of scope:
- Multi-host network topology orchestration.
- Production runtime behavior changes.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | asymmetric parallel lane runs for applied/deferred scenarios | both legs project stable expected reason code and runtime taxonomy version across repeated rounds |
| C-02 | AC-2 | Functional | canonical asymmetric lane profile assertions | lane id ordering and lane-id CSV constant remain deterministic |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions | asymmetric lane markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_asymmetric_parallel_lane_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_asymmetric_parallel_lane_is_deterministic -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_asymmetric_parallel_lane_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Asymmetric parallel lane drift becomes explicitly detectable in tests/docs contracts.
- Docs and tests fail closed on asymmetric marker regressions.
- R45 next-frontier narrative reflects asymmetric parallel lane hardening.
