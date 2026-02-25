# Spec: Issue #5994 - Persist service-api bridge lifecycle state across restart

- Issue: #5994
- Status: Reviewed (agent-authored P1; implementation proceeding)
- Type: story
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5917

## Problem Statement
Bridge routes (`POST /v1/bridge/submit`, `POST /v1/bridge/{bridge_id}/forward`, `GET /v1/bridge/{bridge_id}`) currently return synthetic payload projections and are not backed by durable service-api state.

## Scope
In scope:
- Add durable bridge lifecycle persistence to `ServiceApiMessageStore`.
- Wire bridge submit/forward/query routes through live middleware persistence path.
- Add restart integration coverage for bridge state.
- Preserve existing route contracts and fail-closed missing-resource behavior.

Out of scope:
- External bridge transport/network redesign.
- New cross-network semantics beyond current API contract fields.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: `POST /v1/bridge/submit` persists a durable bridge record and returns deterministic bridge submit payload.
- AC-2: `POST /v1/bridge/{bridge_id}/forward` and `GET /v1/bridge/{bridge_id}` read/update persisted bridge state, and missing IDs fail closed with deterministic `404`/reason code.
- AC-3: Bridge state survives restart and remains queryable with persisted forward markers.
- AC-4: Existing durable relay flow remains green after bridge wiring changes.

## Conformance Cases
- C-01 (Functional, AC-1): Live bridge submit call returns `202` and state file records bridge row.
- C-02 (Functional, AC-2): Live bridge forward/query transitions persisted record; unknown bridge ID yields `404` + `service_api_route_not_found`.
- C-03 (Integration, AC-3): Submit in process A, restart endpoint in process B with same state file, then forward+query preserve state.
- C-04 (Regression, AC-4): Existing relay spool durability integration test remains passing.

## Success Metrics / Observable Signals
- Bridge routes no longer rely on synthetic-only fallback during live middleware execution.
- Restart test proves durable bridge lifecycle persistence.
- Relay durability regression remains green.
