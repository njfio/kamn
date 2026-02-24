# Spec: Issue #5928 - Task: Bound replay guard memory with TTL/capacity eviction

- Issue: #5928
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P0
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5917

## Problem Statement
Replay guard currently grows monotonically with process lifetime.

## Scope
In scope:
- Implement bounded replay cache with TTL and capacity eviction policy.

Out of scope:
- Changes to external auth semantics outside replay-window policy contract.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Replay structure remains memory-bounded under sustained traffic.
- AC-2: Replay attacks within configured window are still rejected.
- AC-3: Load tests verify no unbounded growth and acceptable eviction behavior.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Replay structure remains memory-bounded under sustained traffic.
- C-02 (Functional, AC-2): Verify Replay attacks within configured window are still rejected.
- C-03 (Functional, AC-3): Verify Load tests verify no unbounded growth and acceptable eviction behavior.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: eviction policy and keying behavior
- Functional: replay acceptance/rejection within window
- Integration: service API auth chain with bounded cache
- Regression: unbounded BTreeSet path removed
- Performance: memory and latency under sustained load

## Dependencies
- #5917

