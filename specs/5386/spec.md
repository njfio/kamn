# Issue #5386 Spec

- Title: Task: codify topology-id host-mode-host-pair coherence contracts for live-postgres validation
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Topology-id host-mode mapping and host-pair mapping are explicit separately, but host-mode-to-host-pair coherence remains implicit. Drift that preserves each mapping independently while violating host-mode/host-pair coherence could bypass current contracts.

## Acceptance Criteria
- AC-1: daemon validation tests assert canonical topology-id->host-mode->host-pair coherence schema/version and rows.
- AC-2: integration tests assert topology-id host-mode-host-pair coherence rows remain deterministic and invariant under repeated runs and topology permutations.
- AC-3: `docs/ops/configuration.md` includes explicit `#5386` coherence markers and validation commands.
- AC-4: docs-contract tests fail closed on coherence marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only host-mode-host-pair coherence contracts in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
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
| C-01 | AC-1 | Functional | topology host-mode-host-pair coherence helper contracts | coherence schema version and canonical rows remain explicit/stable |
| C-02 | AC-2 | Integration | repeated/permuted topology fingerprints | extracted topology_id->host_mode->host_pair rows remain identical under permutations |
| C-03 | AC-3/AC-4 | Conformance | docs marker assertions | coherence markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_coherence_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_coherence_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_coherence_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Topology-id host-mode/host-pair coherence drift becomes explicitly detectable.
- Docs and tests fail closed on coherence marker regressions.
- R45 frontier narrative reflects host-mode-host-pair coherence hardening.
