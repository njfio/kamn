# Spec: Issue #6077 - Implement durable send-to-recipient delivery execution path

- Issue: #6077
- Status: Implemented
- Type: task
- Priority: P1
- Area: networking
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6076

## Problem Statement
`POST /v1/messages/send` persists message and relay spool entries, but daemon relay processing still permits a synthetic pass-through path: when no recipient route map is configured, drained spool entries are projected to `relayed` without any recipient delivery execution. This weakens end-to-end delivery guarantees and masks missing runtime wiring.

## Scope
In scope:
- Enforce fail-closed daemon relay semantics when recipient route forwarding is unavailable.
- Preserve pending relay intents in durable spool/state until actual forwarding succeeds.
- Add/adjust integration and regression tests proving durable pending->relayed->delivered lifecycle across daemon/API restart boundaries.
- Document runtime delivery lifecycle and route-map requirements in `docs/architecture/service-api-delivery-flow.md`.

Out of scope:
- Cross-region/internet-scale routing policy.
- New transport protocols beyond existing service API relay route forwarding.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: `POST /v1/messages/send` persists a created message record and durable relay spool entry for recipient-targeted payloads.
- AC-2: Daemon relay ticks without a resolvable recipient forward path do not project message status to `relayed`; pending relay entries remain queued for retry.
- AC-3: When recipient route forwarding is available, daemon relay ticks forward entries to recipient runtime and project sender message state to `relayed` exactly once.
- AC-4: Recipient runtime exposes forwarded message through recipient mailbox/message routes and transitions to `delivered` on recipient read; state survives restart.
- AC-5: Unit, Functional, Integration, and Regression tests enforce non-synthetic behavior and retry durability.

## Conformance Cases
- C-01 (Functional, AC-1): send request returns `202`; sender state file records `messages[message_id].status == "created"`; relay spool contains the message id.
- C-02 (Regression, AC-2): daemon run with no relay route map leaves sender message in `created` and retains spool entry (no synthetic projection).
- C-03 (Integration, AC-3): daemon run with configured recipient route forwards `POST /v1/messages/relay`, drains one pending spool entry, and projects sender message to `relayed`.
- C-04 (Integration, AC-4): recipient mailbox contains forwarded message id; recipient `GET /v1/messages/{id}` returns `delivered`; post-restart query still returns `delivered`.
- C-05 (Regression, AC-5): retry scenario where first daemon attempt fails forwarding and second attempt after recipient availability succeeds with durable state continuity.

## Success Metrics / Observable Signals
- No daemon path can mark recipient-targeted messages as `relayed` without attempted recipient forwarding.
- Pending relay intents persist across failed/offline forwarding attempts and are drained only after successful delivery execution.
- Live integration tests validate send->retry->forward->recipient-observe->restart continuity.
