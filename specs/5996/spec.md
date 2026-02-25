# Spec: Issue #5996 - Persist service-api channel creation state across restart

- Issue: #5996
- Status: Reviewed (agent-authored P1; implementation proceeding)
- Type: story
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5917

## Problem Statement
`POST /v1/channels/create` is synthetic in live service-api middleware and does not persist created channel IDs into durable state.

## Scope
In scope:
- Add durable channel-creation method in `ServiceApiMessageStore`.
- Wire live `POST /v1/channels/create` route to message-store persistence.
- Add restart integration test that verifies persisted channel state.
- Re-run durable relay regression test.

Out of scope:
- Channel membership/permissions model.
- New channel metadata contracts.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: `POST /v1/channels/create` persists channel creation in durable service-api state.
- AC-2: Created channel remains present in durable state across restart and can be queried via channel messages route.
- AC-3: Existing relay durability flow remains green after channel wiring change.

## Conformance Cases
- C-01 (Functional, AC-1): channel create returns `201` and state file records `channel_messages[channel_id]` as present.
- C-02 (Integration, AC-2): create in phase A, restart endpoint, query channel messages in phase B with persisted channel id.
- C-03 (Regression, AC-3): relay durable spool integration test remains passing.

## Success Metrics / Observable Signals
- Live channel creation is no longer synthetic-only.
- Channel ID persistence survives restart.
- Relay durability regression remains green.
