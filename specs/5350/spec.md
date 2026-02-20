# Issue #5350 Spec

- Title: Task: codify role-profile matrix contracts for live-postgres daemon validation
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5348` added bounded load-profile contracts, but live-postgres daemon validation still lacks explicit role-profile matrix contracts across `processor`, `listener`, and `approver` roles. Without deterministic cross-role assertions, reason/taxonomy markers can drift per role.

## Acceptance Criteria
- AC-1: daemon validation tests assert deterministic reason/taxonomy outcomes across canonical applied/deferred role profiles (`processor`, `listener`, `approver`).
- AC-2: canonical role-profile matrix constants and ordering are explicitly asserted in test code.
- AC-3: `docs/ops/configuration.md` includes explicit `#5350` role-profile marker contracts and validation commands.
- AC-4: docs-contract tests fail closed on role-profile marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only role-profile matrix assertions in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
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
| C-01 | AC-1 | Integration | role-profile daemon runs over applied/deferred role variants | each role profile projects stable expected reason code and runtime taxonomy version |
| C-02 | AC-2 | Functional | canonical role-profile matrix assertions | role-profile ordering and profile CSV constants remain deterministic |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions | role-profile markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_role_profile_matrix_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_role_profile_matrix_is_deterministic -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_profile_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Cross-role live-postgres drift becomes explicitly detectable in tests/docs contracts.
- Docs and tests fail closed on role-profile marker regressions.
- R45 next-frontier narrative reflects role-profile contract hardening.
