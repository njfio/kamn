# Issue #5400 Spec

- Title: Task: codify topology lane-fingerprint-hash order-normalization digest contracts for live-postgres validation
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Topology lane-fingerprint-hash order-normalization rows are explicit, but a compact deterministic digest for the normalized row set remains implicit. Drift in canonical sorted row content could evade lightweight integrity checks.

## Acceptance Criteria
- AC-1: daemon validation tests assert canonical topology-id->host-mode->host-pair->lane-set->lane-fingerprint-hash order-normalization digest schema/version and digest row markers.
- AC-2: integration tests assert order-normalization digest rows remain deterministic and invariant under repeated runs and topology permutations while preserving canonical sorted hash rows.
- AC-3: `docs/ops/configuration.md` includes explicit `#5400` order-normalization digest markers and validation commands.
- AC-4: docs-contract tests fail closed on order-normalization digest marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only host/lane fingerprint-hash order-normalization digest contracts in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
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
| C-01 | AC-1 | Functional | topology host/lane fingerprint-hash order-normalization digest helper contracts | digest schema version and canonical digest rows remain explicit/stable |
| C-02 | AC-2 | Integration | repeated/permuted topology fingerprints | extracted order-normalization digest rows remain stable and canonically sorted under permutations |
| C-03 | AC-3/AC-4 | Conformance | docs marker assertions | order-normalization digest markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_is_stable -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Topology host/lane fingerprint-hash order-normalization digest drift becomes explicitly detectable.
- Docs and tests fail closed on digest marker regressions.
- R45 frontier narrative reflects order-normalization digest hardening.
