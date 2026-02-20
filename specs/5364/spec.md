# Issue #5364 Spec

- Title: Task: codify parallel lane topology-scope contracts for live-postgres daemon validation
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5362` hardened fingerprint schema semantics, but topology-scope semantics are still implicit. The bounded validation slice executes same-host lanes today while multi-host distributed lanes remain follow-up work; explicit topology contracts are needed to prevent scope drift.

## Acceptance Criteria
- AC-1: daemon validation tests assert canonical topology-scope schema/version and topology-id set for parallel lane projections.
- AC-2: integration tests assert topology-labeled parallel lane fingerprints remain deterministic and schema-conformant across repeated runs.
- AC-3: `docs/ops/configuration.md` includes explicit `#5364` topology-scope markers and validation commands.
- AC-4: docs-contract tests fail closed on topology-scope marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only topology-scope contracts in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- Ops marker additions in `docs/ops/configuration.md`.
- Docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- R45 review narrative refinement for this increment.

Out of scope:
- True multi-host networking or deployment orchestration.
- Production runtime behavior changes.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | topology schema helper contracts | topology schema version, topology ids CSV, and formatted topology fingerprints remain canonical |
| C-02 | AC-2 | Integration | repeated topology-labeled fingerprint projections | identical sorted fingerprints; each fingerprint remains schema/topology conformant |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions | topology markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_scope_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_scope_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_scope_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Topology-scope drift becomes explicitly detectable in tests/docs contracts.
- Docs and tests fail closed on topology marker regressions.
- R45 next-frontier narrative reflects topology-scope hardening.
