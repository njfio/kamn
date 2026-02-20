# Issue #5346 Spec

- Title: Task: enforce runtime-to-matrix taxonomy bridge contracts for live-postgres daemon validation
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5344` codified canonical scenario ordering and matrix taxonomy markers for the live-postgres daemon validation slice, but runtime taxonomy-version output (`daemon_phase6_runtime_reason_taxonomy_version`) is not yet explicitly bridge-constrained against matrix taxonomy contracts. That leaves drift risk between runtime reason taxonomy and matrix-level taxonomy markers.

## Acceptance Criteria
- AC-1: daemon validation tests assert stable runtime taxonomy-version markers for applied/deferred live-postgres scenarios across repeated runs.
- AC-2: matrix/runtime taxonomy bridge constants are explicitly asserted as deterministic contracts in test code.
- AC-3: `docs/ops/configuration.md` includes explicit `#5346` runtime-to-matrix taxonomy bridge markers and validation commands.
- AC-4: docs-contract tests fail closed on taxonomy-bridge marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only taxonomy bridge assertions in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- Ops marker additions in `docs/ops/configuration.md`.
- Docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- R45 review narrative refinement for this increment.

Out of scope:
- Multi-node/load lane expansion.
- Production runtime behavior changes.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | repeated-run applied/deferred daemon executions | runtime taxonomy version remains stable and equal to canonical runtime taxonomy value |
| C-02 | AC-2 | Functional | taxonomy bridge projection assertions | runtime + matrix taxonomy constants and reason bridge mapping match canonical values |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions | taxonomy bridge markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_matrix_taxonomy_bridge_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_taxonomy_versions_are_stable_across_repeated_runs -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_bridge_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Runtime taxonomy output and matrix taxonomy markers are bridge-constrained by deterministic tests/docs.
- Docs and tests fail closed on runtime-to-matrix taxonomy drift.
- R45 next-frontier narrative reflects this bridge-hardening increment.
