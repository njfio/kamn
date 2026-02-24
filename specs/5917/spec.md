# Spec: Issue #5917 - Story: Runtime Message Delivery and Durable State

- Issue: #5917
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: story
- Priority: P0
- Area: messaging
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent epic: #5915

## Problem Statement
The platform accepts messages but does not perform real end-to-end delivery with durable runtime state.

## Scope
In scope:
- Wire send->relay->delivery with persistent state and bounded replay controls.

Out of scope:
- UI-only changes not related to runtime delivery guarantees.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Message send path produces real recipient delivery transitions, not synthetic counters.
- AC-2: State survives restart and replay protections remain bounded and effective.
- AC-3: Daemon tick loop processes real queued work with deterministic telemetry.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Message send path produces real recipient delivery transitions, not synthetic counters.
- C-02 (Functional, AC-2): Verify State survives restart and replay protections remain bounded and effective.
- C-03 (Functional, AC-3): Verify Daemon tick loop processes real queued work with deterministic telemetry.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: required for touched modules
- Functional: required for behavior paths
- Integration: required where cross-module runtime paths change
- Regression: required for each remediated finding
- Performance: required for hot paths, else justified N/A

## Dependencies
- #5915

