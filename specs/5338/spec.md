# Issue #5338 Spec

- Title: Task: initiate PostgreSQL live integration and daemon runtime e2e validation slice
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
`docs/review/gaps-and-issues-r45.md` identifies the next data-layer milestone as live PostgreSQL integration testing plus daemon runtime end-to-end validation. The repository has both domains covered separately, but lacks a single documented conformance slice that validates both surfaces together in one env-gated lane.

## Acceptance Criteria
- AC-1: Add a daemon runtime validation test that is env-gated on live PostgreSQL URL configuration and, when configured, verifies live adapter connectivity/migrations before asserting daemon Phase-6 runtime markers.
- AC-2: `docs/ops/configuration.md` defines explicit contract markers and command references for the new PostgreSQL live + daemon runtime validation slice.
- AC-3: docs-contract tests enforce the new marker section so drift fails closed.
- AC-4: R45 review narrative is updated to show this next-milestone slice is now initiated under tracked issue `#5338`.
- AC-5: touched suites remain fmt/clippy/test clean under targeted verification.

## Scope
In scope:
- Test-only env-gated live PostgreSQL + daemon runtime validation slice in `kamn-node`.
- Ops documentation contract marker section and docs-contract assertions.
- R45 review doc follow-up marker update for milestone initiation.

Out of scope:
- Full multi-node/load/performance E2E lanes.
- New runtime production behavior or protocol changes.
- New dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | env-gated daemon/live-postgres validation test | with URL configured: adapter connect+migrations succeed and daemon output contains phase6 reason markers |
| C-02 | AC-2 | Functional | ops config marker section for issue `#5338` | section contains schema/taxonomy markers and exact validation commands |
| C-03 | AC-3 | Conformance | docs-contract test for `#5338` section | missing/drifted markers fail test; current section passes |
| C-04 | AC-4 | Governance | R45 review doc next-milestone narrative | includes initiated tracked slice reference to `#5338` |
| C-05 | AC-5 | Quality | fmt/clippy/targeted tests | no formatting/lint/regression failures in touched scope |

## Test Mapping
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_live_postgres_daemon_runtime_validation_slice_markers -- --exact`
- `cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice -- --exact`
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter spec_c01_and_c03_live_adapter_executes_insert_and_lookup_with_session_context -- --exact`
- `rg -n \"#5338|live integration testing against PostgreSQL\" docs/review/gaps-and-issues-r45.md`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-node --tests -- -D warnings`

## Success Metrics
- First cross-surface live-postgres + daemon runtime validation slice exists and is documented.
- Contract markers for the slice are guarded by docs-contract tests.
- R45 review “next milestone” statement moves from generic future intent to tracked initiated execution.
