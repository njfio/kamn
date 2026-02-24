# Spec: Issue #5926 - Task: Wire real end-to-end message delivery from /v1/messages/send to recipient state

- Issue: #5926
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P0
- Area: messaging
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
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
- C-01 (Functional, AC-1): Verify POST /v1/messages/send drives real recipient delivery state transitions.
- C-02 (Functional, AC-2): Verify Delivery survives restart and is queryable via existing API surfaces.
- C-03 (Functional, AC-3): Verify End-to-end integration tests run real processes and verify delivery semantics.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: message state transition handlers
- Functional: send/relay/deliver lifecycle
- Integration: API -> runtime -> recipient delivery
- Regression: synthetic-only pass path removed
- Performance: delivery latency budget for smoke path

## Dependencies
- #5917

