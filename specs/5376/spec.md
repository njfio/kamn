# Issue #5376 Spec

- Title: Task: codify topology-id lane-count mapping contracts for live-postgres validation
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Topology-id to lane-set mapping is explicit, but topology-id lane-count cardinality remains implicit. Cardinality drift could weaken conformance guarantees without explicit mapping contracts.

## Acceptance Criteria
- AC-1: daemon validation tests assert canonical topology-id->lane-count mapping schema/version and rows.
- AC-2: integration tests assert topology-id lane-count mappings remain deterministic and invariant under repeated runs and topology permutations.
- AC-3: `docs/ops/configuration.md` includes explicit `#5376` lane-count mapping markers and validation commands.
- AC-4: docs-contract tests fail closed on lane-count mapping marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only lane-count mapping contracts in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
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
| C-01 | AC-1 | Functional | topology lane-count mapping helper contracts | mapping schema version and canonical rows remain explicit/stable |
| C-02 | AC-2 | Integration | repeated/permuted topology fingerprints | extracted topology_id->lane_count rows remain identical under permutations |
| C-03 | AC-3/AC-4 | Conformance | docs marker assertions | lane-count mapping markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_lane_count_mapping_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_lane_count_mapping_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_lane_count_mapping_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Topology-id to lane-count mapping drift becomes explicitly detectable.
- Docs and tests fail closed on lane-count mapping marker regressions.
- R45 frontier narrative reflects lane-count mapping hardening.
