# Spec: Issue #5845 - Service API Durable Task/Escrow State and Recipient Delivery Semantics

- Issue: #5845
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
Service API runtime persistence currently covers message IDs and channel message lists only. Task and escrow route state transitions remain scaffold responses, and direct recipient delivery semantics are incomplete for message retrieval lifecycle.

## Scope
In scope:
- Persist task lifecycle state for create/read/accept/complete routes.
- Persist escrow lifecycle state for fund/release routes.
- Persist recipient metadata for sent messages and project recipient mailbox mapping.
- Mark recipient-targeted messages as delivered when retrieved by recipient identity.
- Keep existing route/auth surface and response compatibility intact.

Out of scope:
- New public route additions.
- Daemon gossip transport integration.
- Kolme protocol changes.

## Acceptance Criteria
- AC-1: Task lifecycle responses are backed by persisted state across create/read/transition routes.
- AC-2: Escrow lifecycle responses are backed by persisted state across fund/release routes.
- AC-3: Sending a message with recipient metadata records deterministic recipient mailbox projection.
- AC-4: Recipient retrieval updates message status from `created` to `delivered` deterministically.
- AC-5: Existing message and channel response contracts remain backward-compatible for SDK/agent clients.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | create task, accept task, query task | state transitions persisted (`submitted` -> `accepted`) |
| C-02 | AC-1 | Integration | complete task after create | `completed` persisted and query returns `completed` |
| C-03 | AC-2 | Integration | fund escrow then release | escrow state persisted as `released` |
| C-04 | AC-3 | Functional | send message with `recipient_did` | recipient mailbox channel contains message_id |
| C-05 | AC-4 | Regression | recipient queries sent message | status transitions to `delivered` |
| C-06 | AC-5 | Regression | existing message query flow without recipient metadata | status remains `created` and contract fields parse |

## Test Mapping
- `cargo test -p kamn-node integration_service_api_endpoint_lists_channel_messages_from_message_store -- --nocapture`
- `cargo test -p kamn-node integration_service_api_endpoint_persists_message_state_across_restart -- --nocapture`
- `cargo test -p kamn-node integration_service_api_endpoint_persists_task_and_escrow_state_across_routes -- --nocapture`
- `cargo test -p kamn-node integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --nocapture`
- `cargo test -p kamn-node service_api_endpoint_tests:: -- --nocapture`

## Success Metrics / Observable Signals
- Task and escrow route payloads reflect persisted lifecycle state, not static render-only projection.
- Recipient-targeted messages can be discovered via deterministic mailbox channel projection.
- Recipient retrieval deterministically records `delivered` status while preserving legacy parse compatibility.
