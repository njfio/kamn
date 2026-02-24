# Spec: Issue #5933 - Task: Decompose kamn-core into focused crates with phase-1 extraction

- Issue: #5933
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5919

## Problem Statement
kamn-core currently concentrates many domains, increasing change risk and compile blast radius.

## Scope
In scope:
- Extract first tranche into focused crates with stable boundaries and minimal API breakage.

Out of scope:
- Total architecture rewrite in one cycle.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: A documented extraction plan is implemented for phase-1 crate split.
- AC-2: Moved modules compile and pass with preserved behavior contracts.
- AC-3: Public API boundaries and dependency graph are updated and documented.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify A documented extraction plan is implemented for phase-1 crate split.
- C-02 (Functional, AC-2): Verify Moved modules compile and pass with preserved behavior contracts.
- C-03 (Functional, AC-3): Verify Public API boundaries and dependency graph are updated and documented.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: moved module suites stay green
- Functional: cross-crate behavior parity
- Integration: workspace builds/tests with new crate graph
- Regression: API breakage checks for extracted surfaces
- Performance: compile-time telemetry captured

## Dependencies
- #5919

