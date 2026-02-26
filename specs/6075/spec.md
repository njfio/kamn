# Spec: Issue #6075 - Epic: Runtime message delivery end-to-end integration

- Issue: #6075
- Status: Reviewed
- Type: epic
- Priority: P1
- Area: networking
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #1

## Problem Statement
The runtime accepted send requests without fully enforced end-to-end durable delivery semantics across daemon/service boundaries. This epic closes that gap by landing and verifying the first real delivery slice.

## Scope
In scope:
- End-to-end delivery path from service API send ingress to recipient-visible delivery behavior.
- Durable restart continuity for pending and delivered states in the covered local runtime flow.
- Live integration test coverage proving real runtime behavior.

Out of scope:
- Protocol redesign and new external transport families.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Service API ingress produces durable delivery records and recipient-facing state transitions.
- AC-2: Delivery survives restart and resumes without message loss for covered scenarios.
- AC-3: End-to-end integration tests run against live local components and enforce real outcomes.

## Conformance Cases
- C-01 (Functional, AC-1): send path persists `created` + relay spool, forwarding projects `relayed`, recipient read projects `delivered`.
- C-02 (Regression, AC-2): failed-forward/no-route preserves pending state; retry pass after recipient availability succeeds with durable continuity.
- C-03 (Integration, AC-3): live runtime tests cover send->relay->mailbox->message-read flows with restart checks.

## Success Metrics / Observable Signals
- Epic-level delivery path is no longer synthetic for the covered local runtime slice.
- Parent story/task ACs are merged and traceable (`#6076`, `#6077`, PR #6078).
- Validation gates and mutation checks are green on current `main`.
