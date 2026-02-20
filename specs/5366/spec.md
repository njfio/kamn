# Issue #5366 Spec

- Title: Task: add topology permutation-invariance contracts for live-postgres parallel lane validation
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5364` codified topology-scope schema contracts, but topology-profile ordering semantics are still implicit. Permutation drift in topology profile ordering could bypass explicit contract checks without dedicated invariance tests.

## Acceptance Criteria
- AC-1: daemon validation tests assert canonical topology permutation ids and permutation-contract markers.
- AC-2: integration tests assert topology-labeled fingerprint bundles remain deterministic under canonical topology permutations.
- AC-3: `docs/ops/configuration.md` includes explicit `#5366` topology permutation markers and validation commands.
- AC-4: docs-contract tests fail closed on topology permutation marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only topology permutation contracts in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
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
| C-01 | AC-1 | Functional | topology permutation helper contracts | topology permutation ids and canonical permuted profile order remain explicit and stable |
| C-02 | AC-2 | Integration | repeated topology-labeled fingerprints under permutations | equivalent sorted topology fingerprints across canonical permutations |
| C-03 | AC-3/AC-4 | Conformance | docs marker assertions | topology permutation markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_permutation_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_permutations_are_invariant -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_parallel_lane_topology_permutation_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Topology-order drift becomes explicitly detectable under deterministic permutations.
- Docs and tests fail closed on topology permutation marker regressions.
- R45 frontier narrative reflects topology permutation hardening.
