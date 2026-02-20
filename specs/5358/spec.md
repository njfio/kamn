# Issue #5358 Spec

- Title: Task: add parallel lane order-invariance contracts for live-postgres daemon validation
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5356` codified deterministic bounded asymmetric parallel lanes, but does not explicitly enforce order-invariance. Reordering lane execution should not alter reason/taxonomy outcomes across the bounded same-host parallel matrix.

## Acceptance Criteria
- AC-1: daemon validation tests assert deterministic order-invariance for bounded parallel/asymmetric lanes across applied/deferred scenarios.
- AC-2: canonical lane-order fingerprints are explicitly asserted in test code for both baseline and permuted execution orders.
- AC-3: `docs/ops/configuration.md` includes explicit `#5358` order-invariance marker contracts and validation commands.
- AC-4: docs-contract tests fail closed on order-invariance marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only order-invariance assertions in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
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
| C-01 | AC-1 | Integration | baseline vs permuted lane-order executions | normalized lane fingerprints (reason+taxonomy) remain equivalent |
| C-02 | AC-2 | Functional | deterministic lane fingerprint projection helper assertions | canonical fingerprint ordering remains stable and reproducible |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions | order-invariance markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_order_invariance_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_order_is_invariant -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_order_invariance_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Lane-order drift in bounded same-host parallel matrices becomes explicitly detectable in tests/docs contracts.
- Docs and tests fail closed on order-invariance marker regressions.
- R45 next-frontier narrative reflects order-invariance hardening.
