# Issue #5370 Spec

- Title: Task: codify topology host-pair directionality contracts for live-postgres lane validation
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5368` codified required host-pair ids, but host-pair directionality semantics remain implicit. Drift that swaps host-pair direction (`host_b->host_a`) could weaken topology contracts without explicit directionality checks.

## Acceptance Criteria
- AC-1: daemon validation tests assert canonical host-pair directionality schema/version and extraction rule markers.
- AC-2: integration tests assert host-pair directionality remains deterministic and non-commutative under repeated runs and topology permutations.
- AC-3: `docs/ops/configuration.md` includes explicit `#5370` directionality markers and validation commands.
- AC-4: docs-contract tests fail closed on directionality marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only directionality contracts in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- Ops marker additions in `docs/ops/configuration.md`.
- Docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- R45 review narrative refinement for this increment.

Out of scope:
- True multi-host orchestration.
- Production runtime behavior changes.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | directionality helper contracts | directionality schema version + extraction rule remain canonical (`host_a->host_b`) |
| C-02 | AC-2 | Integration | repeated/permuted topology fingerprints | extracted host-pair ids are stable and never reverse canonical distributed host-pair direction |
| C-03 | AC-3/AC-4 | Conformance | docs marker assertions | directionality markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_directionality_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_directionality_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Host-pair directionality drift becomes explicitly detectable.
- Docs and tests fail closed on directionality marker regressions.
- R45 frontier narrative reflects directionality hardening.
