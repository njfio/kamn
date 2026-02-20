# Issue #5360 Spec

- Title: Task: add multi-permutation order-invariance contracts for live-postgres parallel lanes
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5358` validated order-invariance for baseline vs reverse lane order only. Deterministic rotation/interleaving permutations are not yet enforced, leaving potential ordering-drift blind spots.

## Acceptance Criteria
- AC-1: daemon validation tests assert deterministic lane-fingerprint invariance across multiple deterministic permutations for symmetric and asymmetric lane sets.
- AC-2: permutation set identifiers and ordering are explicitly asserted as canonical contracts in test code.
- AC-3: `docs/ops/configuration.md` includes explicit `#5360` multi-permutation marker contracts and validation commands.
- AC-4: docs-contract tests fail closed on multi-permutation marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only multi-permutation invariance assertions in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
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
| C-01 | AC-1 | Integration | baseline/reverse/rotate/interleaved permutations over lane sets | sorted lane fingerprints remain equivalent across permutations |
| C-02 | AC-2 | Functional | canonical permutation-id assertions | permutation-id CSV and lane-set coverage remain deterministic |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions | multi-permutation markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_permutation_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_permutations_are_invariant -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_permutation_invariance_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Order-sensitivity drift under deterministic permutations becomes explicitly detectable in tests/docs contracts.
- Docs and tests fail closed on permutation marker regressions.
- R45 next-frontier narrative reflects permutation invariance hardening.
