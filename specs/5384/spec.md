# Issue #5384 Spec

- Title: Task: codify topology-id host-pair-cardinality coherence contracts for live-postgres validation
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Topology-id host-pair mapping and host-cardinality mapping are explicit separately, but host-pair-to-cardinality coherence remains implicit. Drift that preserves each mapping independently while violating host-pair cardinality coherence could bypass current contracts.

## Acceptance Criteria
- AC-1: daemon validation tests assert canonical topology-id->host-pair->host-cardinality coherence schema/version and rows.
- AC-2: integration tests assert topology-id host-pair-cardinality coherence rows remain deterministic and invariant under repeated runs and topology permutations.
- AC-3: `docs/ops/configuration.md` includes explicit `#5384` coherence markers and validation commands.
- AC-4: docs-contract tests fail closed on coherence marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only host-pair-cardinality coherence contracts in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
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
| C-01 | AC-1 | Functional | topology host-pair-cardinality coherence helper contracts | coherence schema version and canonical rows remain explicit/stable |
| C-02 | AC-2 | Integration | repeated/permuted topology fingerprints | extracted topology_id->host_pair->host_cardinality rows remain identical under permutations |
| C-03 | AC-3/AC-4 | Conformance | docs marker assertions | coherence markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_cardinality_coherence_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pair_cardinality_coherence_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_cardinality_coherence_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Topology-id host-pair/cardinality coherence drift becomes explicitly detectable.
- Docs and tests fail closed on coherence marker regressions.
- R45 frontier narrative reflects host-pair-cardinality coherence hardening.
