# Spec: Issue #5863 - Daemon Relay Drain Lifecycle Projection

- Issue: #5863
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Relay spool drain currently clears queued entries but does not project drained message IDs back into durable Service API lifecycle state. This leaves daemon execution disconnected from persisted message lifecycle progression.

## Scope
In scope:
- Project drained relay message IDs from `created` to `relayed` in Service API state file.
- Allow recipient retrieval to transition both `created` and `relayed` to `delivered`.
- Add tests covering projection and recipient retrieval continuity.

Out of scope:
- Cross-node gossip routing.
- Service API wire/protocol schema changes.
- Kolme upstream changes.

## Acceptance Criteria
- AC-1: Daemon relay drain marks persisted message records `created` -> `relayed`.
- AC-2: Recipient message retrieval transitions `relayed` -> `delivered`.
- AC-3: Relay projection is idempotent for already `relayed`/`delivered` records.
- AC-4: Unit/integration/regression checks for projection and retrieval behavior pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | state file with `created` message + matching relay spool entry, daemon execution | message status becomes `relayed` after drain |
| C-02 | AC-2 | Integration/Regression | message record status `relayed`, recipient `GET /v1/messages/{id}` | response status `delivered` and persisted state updated |
| C-03 | AC-3 | Unit/Regression | state file records already `relayed`/`delivered` with relay IDs | no invalid transition; operation remains deterministic |
| C-04 | AC-4 | Verify | scoped tests + fmt/clippy checks | all pass |

## Test Mapping
- `cargo test -p kamn-node main_tests::runtime_tests::integration_runtime_daemon_relay_drain_projects_message_state_to_relayed -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_recipient_query_promotes_relayed_to_delivered -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::regression_runtime_daemon_relay_state_projection_is_idempotent_for_relayed_messages -- --exact`

## Success Metrics / Observable Signals
- Relay drain emits deterministic counts for drained and state-projected messages.
- Persisted lifecycle reflects daemon relay execution (`relayed`).
- Recipient retrieval remains stable and completes to `delivered`.
