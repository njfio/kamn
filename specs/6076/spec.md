# Spec: Issue #6076 - Story: Send API performs real durable recipient delivery

- Issue: #6076
- Status: Reviewed
- Type: story
- Priority: P1
- Area: networking
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6075

## Problem Statement
`POST /v1/messages/send` previously accepted writes without proving end-to-end durable recipient delivery behavior across runtime boundaries. The story requires real delivery execution, fail-closed relay semantics, and restart durability.

## Scope
In scope:
- Runtime wiring from send API ingress to durable recipient delivery lifecycle.
- Recipient mailbox/message route observability for delivery state.
- Restart continuity for pending and delivered state in covered scenarios.
- AC mapping to merged implementation and tests from #6077 / PR #6078.

Out of scope:
- Internet-scale routing and benchmark expansion.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: `POST /v1/messages/send` enqueues durable delivery state.
- AC-2: Recipient delivery state is observable via runtime paths used in production.
- AC-3: Restart/recovery preserves pending delivery state for covered scenarios.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): send request returns `202`, persists `created` sender message state, and writes relay spool entry.
- C-02 (Integration, AC-2): configured relay forwarding produces sender-side `relayed` and recipient-observable message state.
- C-03 (Regression, AC-3): no-route/failed-forward daemon pass preserves pending sender state and durable spool entry; retry pass after recipient availability succeeds.
- C-04 (Conformance, AC-4): mapped runtime/service tests plus full `kamn-node` suite and in-diff mutation gate pass.

## Success Metrics / Observable Signals
- No synthetic relay projection on missing route-map paths.
- Durable retry continuity across daemon/API restarts.
- Story ACs are traceable to merged tests and green validation gates.
