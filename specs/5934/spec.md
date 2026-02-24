# Spec: Issue #5934 - Task: Reduce shell/python surface below policy ceiling and improve governance ratio

- Issue: #5934
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P1
- Area: governance
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5919

## Problem Statement
Shell LOC exceeds hard ceiling and governance artifact churn dominates commit mix.

## Scope
In scope:
- Retire/merge redundant scripts, migrate high-value script logic to Rust lanes, and enforce ratio gates.

Out of scope:
- Removing essential CI safeguards without replacement.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Shell LOC is reduced below configured hard ceiling with measurable trend improvement.
- AC-2: Shell-to-Rust ratio improves and regression gates enforce non-backsliding.
- AC-3: Governance-commit ratio target (<50%) is tracked with policy checks and visible telemetry.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Shell LOC is reduced below configured hard ceiling with measurable trend improvement.
- C-02 (Functional, AC-2): Verify Shell-to-Rust ratio improves and regression gates enforce non-backsliding.
- C-03 (Functional, AC-3): Verify Governance-commit ratio target (<50%) is tracked with policy checks and visible telemetry.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: policy checker tests
- Functional: replacement Rust lanes for retired scripts
- Integration: CI workflows pass with reduced script surface
- Regression: shell ceiling and ratio checks enforced
- Performance: CI runtime budget remains within target

## Dependencies
- #5919

