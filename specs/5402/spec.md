# Issue #5402 Spec

- Title: Task: decompose daemon_tests.rs by extracting live-postgres fixtures/helpers into submodule structure
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
`crates/kamn-node/src/main_tests/daemon_tests.rs` regrew into a monolith that mixes daemon runtime contract tests with extensive live-postgres fixture topology/helper logic. This blocks maintainability and makes future extraction work harder.

## Acceptance Criteria
- AC-1: live-postgres fixture/topology/hash constants, models, and helper projections are extracted into `crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs` and wired via `mod` + imports from `daemon_tests.rs`.
- AC-2: representative existing live-postgres test commands keep the same test path prefix (`main_tests::daemon_tests::...`) and pass unchanged.
- AC-3: `daemon_tests.rs` line count is reduced by a measurable phase-1 amount (target: <= 4300 lines after extraction).
- AC-4: `docs/ops/configuration.md` includes explicit `#5402` decomposition markers, including extracted module path and test-path-stability contract.
- AC-5: docs-contract tests fail closed on `#5402` decomposition marker drift.
- AC-6: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only decomposition of live-postgres fixtures/helpers from `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- New submodule file under `crates/kamn-node/src/main_tests/daemon_tests/`.
- Ops docs markers + docs-contract assertions for phase-1 decomposition contracts.
- R45 review narrative refinement for this increment.

Out of scope:
- Production runtime behavior changes.
- Renaming existing documented test command paths.
- Full decomposition of all daemon tests in one issue.
- Branch cleanup wave.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | daemon test module wiring | extracted live-postgres fixtures/helpers compile and remain available to daemon tests |
| C-02 | AC-2 | Integration | unchanged exact daemon test commands | representative live-postgres tests pass with original `main_tests::daemon_tests::...` path |
| C-03 | AC-3 | Functional | line-count measurement (`wc -l`) | `daemon_tests.rs` reduced to <= 4300 lines |
| C-04 | AC-4/AC-5 | Conformance | docs marker assertions | `#5402` decomposition markers present; drift fails closed |
| C-05 | AC-6 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_daemon_tests_live_postgres_fixture_decomposition_markers -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_is_stable -- --exact`
- `wc -l crates/kamn-node/src/main_tests/daemon_tests.rs`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- `daemon_tests.rs` drops below 4300 lines in this phase-1 extraction.
- Live-postgres validation test path commands remain stable.
- Decomposition contracts become explicit and fail-closed in docs/docs-contract tests.
