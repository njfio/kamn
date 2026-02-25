# Spec: Issue #5990 - Expand runtime/data-layer durable wiring beyond baseline relay flow

- Issue: #5990
- Status: Reviewed (agent-authored P1; implementation proceeding)
- Type: story
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5917

## Problem Statement
The service API currently wires message relay durability, task state, and escrow state through `ServiceApiMessageStore`, but content lifecycle routes remain synthetic fallback responses (`register/expire/tombstone/get`) without durable persistence or restart-safe behavior.

## Scope
In scope:
- Wire content lifecycle routes through `ServiceApiMessageStore` durable state.
- Persist content records and lifecycle transitions (`retained` -> `expired` -> `tombstoned`) in the existing state snapshot file.
- Add integration coverage proving restart-safe content lifecycle behavior.
- Re-run baseline relay coverage to prevent regression in existing cross-node relay flow.

Out of scope:
- Bridge route durable wiring.
- Protocol redesign for content schema.
- Full database backend migration beyond existing service-api state persistence path.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: `POST /v1/content/register` persists a content record in durable service-api state and returns a stable content payload contract.
- AC-2: `POST /v1/content/{id}/expire`, `POST /v1/content/{id}/tombstone`, and `GET /v1/content/{id}` operate on persisted content records, fail closed on missing IDs, and return deterministic lifecycle/redaction fields.
- AC-3: Content lifecycle state survives service API restart when backed by the configured state file.
- AC-4: Existing cross-node relay durable flow remains green after the content lifecycle wiring expansion.

## Conformance Cases
- C-01 (Functional, AC-1): Live endpoint `POST /v1/content/register` returns `201` and writes content record to state file.
- C-02 (Functional, AC-2): Live endpoint lifecycle transitions and query reflect persisted state changes; missing IDs return deterministic `404` route-not-found envelope.
- C-03 (Integration, AC-3): Register+transition in process A, restart endpoint in process B with same state file, then query preserves lifecycle state.
- C-04 (Regression, AC-4): Existing durable relay path test remains passing.

## Success Metrics / Observable Signals
- Content lifecycle routes are no longer synthetic-only in live middleware flow.
- Restart integration test passes with persisted lifecycle state.
- Relay durability regression test remains green.
