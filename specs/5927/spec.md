# Spec: Issue #5927 - Task: Replace synthetic daemon tick loop behavior with real queue processing

- Issue: #5927
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P0
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5917

## Problem Statement
Current tick loop increments counters and executes hardcoded scenarios without real message processing.

## Scope
In scope:
- Implement queue polling, work dispatch, and lifecycle updates per tick.

Out of scope:
- Daemon redesign unrelated to message processing correctness.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Tick loop processes queued message work and updates durable state.
- AC-2: Telemetry reflects real processed work, not fabricated counters.
- AC-3: Runtime tests prove processing continues across ticks and restart boundaries.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Tick loop processes queued message work and updates durable state.
- C-02 (Functional, AC-2): Verify Telemetry reflects real processed work, not fabricated counters.
- C-03 (Functional, AC-3): Verify Runtime tests prove processing continues across ticks and restart boundaries.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: tick scheduler and queue worker behavior
- Functional: daemon processes queued messages
- Integration: daemon + API + store
- Regression: synthetic counter-only mode removed from production path
- Performance: bounded per-tick processing time

## Dependencies
- #5917

