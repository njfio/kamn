# Spec: Issue #5998 - Persist service-api agent profile query state across restart

- Issue: #5998
- Status: Implemented
- Type: story
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5917

## Problem Statement
`GET /v1/agents/{agent_did}` currently resolves through synthetic fallback rendering and is not wired to durable service-api state.

## Scope
In scope:
- Add durable agent-profile snapshot map in `ServiceApiMessageStore`.
- Wire live agent-profile query route through message-store persistence path.
- Add restart integration coverage proving agent profile query durability.
- Re-run relay durability integration regression lane.

Out of scope:
- Reputation algorithm redesign.
- Expanded profile schema beyond existing fields.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: `GET /v1/agents/{agent_did}` returns `ServiceApiAgentGetBody` from durable message-store path.
- AC-2: Queried agent profile persists across restart in service-api state file.
- AC-3: Existing relay durability integration remains green after agent-profile wiring change.

## Conformance Cases
- C-01 (Functional, AC-1): live query returns `200` with deterministic `did` + `reputation_score`.
- C-02 (Integration, AC-2): query profile in phase A, restart endpoint with same state file, query again in phase B with persisted state assertion.
- C-03 (Regression, AC-3): relay spool durability integration test remains passing.

## Success Metrics / Observable Signals
- Live agent-profile route no longer depends on synthetic-only fallback path.
- Agent profile state is durable across restart.
- Relay durability baseline remains unchanged.
