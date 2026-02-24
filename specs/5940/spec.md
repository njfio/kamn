# Spec: Issue #5940 - Task: Deepen crypto and data-layer module test depth (M6-M11 + DM/Group crypto)

- Issue: #5940
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5920

## Problem Statement
Crypto modules and data-layer M6-M11 currently lack sufficient direct unit depth.

## Scope
In scope:
- Add comprehensive positive/negative/edge-case test suites for DM/group crypto and M6-M11 modules.

Out of scope:
- Replacing integration tests with unit tests only.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: DM/group crypto modules have robust negative/tamper/vector test coverage.
- AC-2: M6-M11 modules have baseline unit suites for each public behavior contract.
- AC-3: Coverage reports show material increase on previously untested modules.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify DM/group crypto modules have robust negative/tamper/vector test coverage.
- C-02 (Functional, AC-2): Verify M6-M11 modules have baseline unit suites for each public behavior contract.
- C-03 (Functional, AC-3): Verify Coverage reports show material increase on previously untested modules.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: module-complete suites for M6-M11 and crypto modules
- Functional: scenario-level behavior for affected modules
- Integration: selected end-to-end checks for wired data-layer paths
- Regression: previously reported blind spots covered
- Performance: N/A unless hotspot tests added

## Dependencies
- #5920

