# Issue #5342 Spec

- Title: Task: add scenario-matrix stability contracts for live-postgres daemon validation
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issues `#5338` and `#5340` introduced and hardened an env-gated PostgreSQL live-integration + daemon runtime validation slice. The slice still lacks a single scenario-matrix contract that verifies env-unset, applied, and deferred outcomes together and guards repeated-run reason stability.

## Acceptance Criteria
- AC-1: daemon validation tests include deterministic scenario-matrix coverage for env-unset gate decision and env-set applied/deferred runtime outcomes.
- AC-2: live-gated applied/deferred scenarios assert repeated-run reason stability for deterministic projection behavior.
- AC-3: `docs/ops/configuration.md` includes explicit scenario-matrix/stability marker contracts and command references for issue `#5342`.
- AC-4: docs-contract tests fail closed when new matrix/stability markers drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only scenario-matrix and repeated-run stability coverage in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- Ops marker contract additions in `docs/ops/configuration.md`.
- Docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- R45 review next-frontier narrative refinement for this stabilization increment.

Out of scope:
- Multi-node distributed orchestration.
- Load/performance benchmarking expansion.
- Production runtime behavior changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | env matrix gate test with unset/primary/fallback/trimmed env inputs | deterministic gate reason + selected URL outcomes |
| C-02 | AC-1/AC-2 | Integration | live URL configured + applied/deferred daemon args run repeatedly | stable reason markers per scenario across repeated runs |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions for `#5342` | matrix/stability markers and commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_env_matrix_contract_is_deterministic -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_stability_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Scenario-matrix and repeated-run stability for the live-postgres daemon slice are explicitly tested.
- Ops/docs contracts for matrix/stability are enforced by docs-contract tests.
- R45 next-frontier narrative reflects this incremental stabilization beyond `#5340`.
