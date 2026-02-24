# Spec: Issue #5866 - Service API Durable Persistence Continuity

- Issue: #5866
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Service API mutation paths require explicit durable persistence continuity guarantees across restart for message, channel-message index, task, and escrow state.

## Scope
In scope:
- Verify and complete durable write + reload continuity across Service API mutation families.
- Add conformance/regression coverage for restart continuity and unchanged-state behavior.
- Keep behavior fail-closed when state payload/file is invalid.

Out of scope:
- Multi-node relay networking.
- CI workflow changes.

## Acceptance Criteria
- AC-1: Mutation endpoints persist durable state under configured state-file path.
- AC-2: Restart with same state file reloads prior state deterministically.
- AC-3: When no mutation occurs, state writes are not spuriously rewritten.
- AC-4: Scoped unit/integration/regression tests map and pass for AC-1..AC-3.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | message/task/escrow/channel mutation requests | state file reflects durable updates |
| C-02 | AC-2 | Integration/Regression | restart using same state file | pre-restart state available post-restart |
| C-03 | AC-3 | Unit/Regression | retrieval-only operations | no state rewrite on no-op path |
| C-04 | AC-4 | Verify | scoped test/quality gates | all pass |

## Test Mapping
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_persists_task_and_escrow_state_across_routes -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_persists_task_and_escrow_state_across_restart -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::regression_service_api_endpoint_rejects_unknown_task_and_escrow_resource_transitions -- --exact`
- `cargo test -p kamn-node service_api_endpoint::tests::service_api_relay_projection_does_not_rewrite_when_no_records_promoted -- --exact`

## Success Metrics / Observable Signals
- Durable state persists across process restarts in covered operation families.
- No-op reads do not trigger unnecessary write churn.
