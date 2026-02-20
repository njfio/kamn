# Issue #5348 Spec

- Title: Task: add bounded load-profile matrix contracts for live-postgres daemon validation
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
The live-postgres daemon validation slice now covers gate/deferred behavior, scenario matrix ordering, and runtime-to-matrix taxonomy bridge contracts, but runtime load-profile diversity remains underconstrained. Deterministic reason/taxonomy outcomes are not explicitly asserted across bounded max-ticks/tick-interval profile variations.

## Acceptance Criteria
- AC-1: daemon validation tests assert deterministic reason/taxonomy outcomes across a canonical bounded load-profile matrix for applied and deferred live-postgres scenarios.
- AC-2: canonical load-profile matrix constants and ordering are explicitly asserted in test code.
- AC-3: `docs/ops/configuration.md` includes explicit `#5348` load-profile marker contracts and validation commands.
- AC-4: docs-contract tests fail closed on load-profile marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only bounded load-profile matrix assertions in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- Ops marker additions in `docs/ops/configuration.md`.
- Docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- R45 review narrative refinement for this increment.

Out of scope:
- Multi-node distributed topology lanes.
- Production runtime behavior changes.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | bounded load-profile daemon runs over applied/deferred scenarios | each profile projects stable expected reason code and runtime taxonomy version |
| C-02 | AC-2 | Functional | canonical profile matrix projection assertions | profile ordering and profile CSV constants remain deterministic |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions | load-profile markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_load_profile_matrix_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_load_profile_matrix_is_deterministic -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_load_profile_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Bounded live-postgres load-profile drift becomes explicitly detectable in tests/docs contracts.
- Docs and tests fail closed on load-profile marker regressions.
- R45 next-frontier narrative reflects load-profile contract hardening.
