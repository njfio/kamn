# Spec: Issue #5926 - Task: Wire real end-to-end message delivery from /v1/messages/send to recipient state

- Issue: #5926
- Status: Implemented
- Type: task
- Priority: P0
- Area: messaging
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5917

## Problem Statement
Core product promise (secure agent-to-agent messaging) is currently not realized end-to-end.

## Scope
In scope:
- Route API send requests through runtime queue/transport and persist lifecycle states through delivery.

Out of scope:
- New protocol features outside existing message lifecycle contracts.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: POST /v1/messages/send drives real recipient delivery state transitions.
- AC-2: Delivery survives restart and is queryable via existing API surfaces.
- AC-3: End-to-end integration tests run real processes and verify delivery semantics.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): `main_tests::service_api_endpoint_tests::integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract` verifies `/v1/messages/send` -> relay spool -> daemon projection -> recipient delivery state transitions.
- C-02 (Functional, AC-2): `main_tests::service_api_endpoint_tests::integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract` verifies relay/delivery state survives restart and remains queryable.
- C-03 (Integration, AC-3): `main_tests::runtime_tests::integration_runtime_daemon_relay_drain_projects_message_state_to_relayed` and `main_tests::runtime_tests::integration_runtime_daemon_processes_relay_entries_arriving_during_tick_loop` verify real daemon/runtime processing against durable spool + state files.
- C-04 (Verify, AC-4): `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --exact`, `cargo test -p kamn-node --bin kamn-node main_tests::service_api_endpoint_tests::regression_service_api_endpoint_recipient_query_requires_relayed_state_before_delivery -- --exact`, `cargo test -p kamn-node --bin kamn-node main_tests::runtime_tests::integration_runtime_daemon_relay_drain_projects_message_state_to_relayed -- --exact`, `cargo test -p kamn-node --bin kamn-node main_tests::runtime_tests::integration_runtime_daemon_processes_relay_entries_arriving_during_tick_loop -- --exact`, `cargo fmt --check`, and `cargo clippy -p kamn-node --bin kamn-node -- -D warnings` pass.

## Success Metrics / Observable Signals
- `/v1/messages/send` produces durable relay entries and recipient mailbox linkage.
- Daemon runtime drains relay spool entries and projects durable state from `created` -> `relayed`.
- Recipient retrieval deterministically advances `relayed` -> `delivered` and preserves delivery status across restart.
- Scoped verification commands pass with no conformance regressions.


## Required Test Categories
- Unit: message state transition handlers
- Functional: send/relay/deliver lifecycle
- Integration: API -> runtime -> recipient delivery
- Regression: synthetic-only pass path removed
- Performance: delivery latency budget for smoke path

## Dependencies
- #5917
