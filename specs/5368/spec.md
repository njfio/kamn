# Issue #5368 Spec

- Title: Task: codify topology host-pair contracts for live-postgres parallel lane validation
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5366` hardened topology permutation invariance, but host-pair semantics remain implicit in topology-labeled fingerprints. Host-pair drift could silently blur same-host vs distributed-labeled contract coverage.

## Acceptance Criteria
- AC-1: daemon validation tests assert canonical topology host-pair schema/version and required host-pair ids.
- AC-2: integration tests assert host-pair-labeled topology fingerprints remain deterministic and permutation-invariant across repeated runs.
- AC-3: `docs/ops/configuration.md` includes explicit `#5368` host-pair markers and validation commands.
- AC-4: docs-contract tests fail closed on host-pair marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only host-pair contracts in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
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
| C-01 | AC-1 | Functional | host-pair helper contracts | schema version, host-pair ids CSV, and host-pair extraction remain canonical |
| C-02 | AC-2 | Integration | repeated+permuted topology fingerprints | host-pair mappings remain deterministic and invariant under permutations |
| C-03 | AC-3/AC-4 | Conformance | docs marker assertions | host-pair markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_pair_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_pairs_are_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_pair_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Host-pair drift becomes explicitly detectable for same-host and distributed-labeled topology fingerprints.
- Docs and tests fail closed on host-pair marker regressions.
- R45 frontier narrative reflects topology host-pair hardening.
