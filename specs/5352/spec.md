# Issue #5352 Spec

- Title: Task: add two-node role-pair matrix contracts for live-postgres daemon validation
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
Issue `#5350` codified deterministic single-node role-profile behavior for live-postgres daemon validation, but ordered two-node role-pair sequences are not yet enforced by explicit contracts. Drift in reason/taxonomy outputs across role-pair handoff-style runs would currently be underconstrained.

## Acceptance Criteria
- AC-1: daemon validation tests assert deterministic reason/taxonomy outcomes across a canonical ordered two-node role-pair matrix for applied/deferred scenarios.
- AC-2: canonical role-pair matrix constants and ordering are explicitly asserted in test code.
- AC-3: `docs/ops/configuration.md` includes explicit `#5352` role-pair marker contracts and validation commands.
- AC-4: docs-contract tests fail closed on role-pair marker drift.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only two-node role-pair matrix assertions in `crates/kamn-node/src/main_tests/daemon_tests.rs`.
- Ops marker additions in `docs/ops/configuration.md`.
- Docs-contract assertions in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- R45 review narrative refinement for this increment.

Out of scope:
- True parallel multi-node networking lanes.
- Production runtime behavior changes.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | ordered role-pair daemon runs over applied/deferred pair variants | each pair leg projects stable expected reason code and runtime taxonomy version |
| C-02 | AC-2 | Functional | canonical role-pair matrix assertions | role-pair ordering and pair-id CSV constant remain deterministic |
| C-03 | AC-3/AC-4 | Conformance | docs marker section assertions | role-pair markers/commands present; drift fails closed |
| C-04 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures |

## Test Mapping
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_role_pair_matrix_contract_is_canonical -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_role_pair_matrix_is_deterministic -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_matrix_role_pair_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- Ordered two-node role-pair drift becomes explicitly detectable in tests/docs contracts.
- Docs and tests fail closed on role-pair marker regressions.
- R45 next-frontier narrative reflects role-pair contract hardening.
