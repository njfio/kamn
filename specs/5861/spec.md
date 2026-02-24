# Spec: Issue #5861 - Service API Relay Spool to Daemon Drain Integration

- Issue: #5861
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
`kamn-node` Service API persists message/task/escrow state, but recipient-directed message sends do not have a concrete relay handoff into daemon runtime. Full-mode API and daemon lanes run concurrently with no durable relay bridge.

## Scope
In scope:
- Add deterministic Service API relay spool path resolution.
- Append recipient-addressed `POST /v1/messages/send` operations into durable relay spool entries.
- Drain the relay spool in daemon runtime with deterministic cleanup and logging.
- Add targeted tests for relay enqueue and relay drain contracts.

Out of scope:
- Cross-node gossip transport implementation.
- Protocol/wire-format response changes.
- Kolme upstream repository modifications.

## Acceptance Criteria
- AC-1: Recipient-addressed message sends append deterministic durable relay entries to relay spool.
- AC-2: Daemon runtime drains relay spool entries and clears consumed backlog deterministically.
- AC-3: Existing message send/query response schemas remain unchanged.
- AC-4: Unit/integration/regression tests validate enqueue, drain, and idempotent empty-drain behavior.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Integration | Send message with `recipient_did` through Service API | relay spool file contains one relay entry with matching `message_id` and recipient |
| C-02 | AC-2 | Integration | Execute daemon runtime with relay spool containing entries | daemon drains entries and truncates/clears spool backlog |
| C-03 | AC-3 | Regression | Existing send/query route requests | status codes and JSON schema remain unchanged |
| C-04 | AC-4 | Verify | Scoped `cargo test`, `cargo clippy`, `cargo fmt --check` | all pass |

## Test Mapping
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_enqueues_recipient_relays_to_durable_spool -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::integration_runtime_daemon_drains_service_api_relay_spool_entries -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::regression_runtime_daemon_relay_spool_drain_is_idempotent_when_empty -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --exact`

## Success Metrics / Observable Signals
- Recipient-targeted sends produce durable relay spool evidence.
- Daemon runtime consumes and clears relay spool backlog on execution.
- No schema or status-code regressions on existing Service API operations.
