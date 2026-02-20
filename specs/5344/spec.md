# Issue #5344 Spec

- Title: Task: codify taxonomy and canonical ordering contracts for live-postgres matrix slice
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5342` added scenario-matrix coverage and repeated-run stability for the live-postgres daemon validation slice, but taxonomy/version markers and canonical scenario ordering contracts are not yet explicitly codified and enforced.

## Acceptance Criteria
- AC-1: daemon matrix tests assert canonical scenario ordering with deterministic row projections for env-unset/applied/deferred states.
- AC-2: matrix reason taxonomy version and reason-codes CSV are exposed as explicit deterministic contracts in tests/docs.
- AC-3: `docs/ops/configuration.md` includes explicit `#5344` taxonomy/ordering markers and command references.
- AC-4: docs-contract tests fail closed on taxonomy/ordering marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only canonical matrix ordering/taxonomy assertions in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- Ops marker contract additions in `docs/ops/configuration.md`.
- Docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- R45 review narrative refinement for this increment.

Out of scope:
- Multi-node/load-lane expansion.
- Production runtime behavior changes.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | canonical matrix row projection test | rows ordered `env_unset,env_set_no_shutdown,env_set_shutdown` with deterministic reasons |
| C-02 | AC-2 | Functional | taxonomy marker projection assertions | taxonomy version and reason-codes CSV match canonical values |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions | taxonomy/ordering markers and commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_taxonomy_ordering_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Canonical matrix ordering and taxonomy markers become explicit deterministic contracts.
- Docs and tests enforce the same taxonomy/order semantics fail-closed.
- R45 frontier narrative reflects this contract-hardening increment.
