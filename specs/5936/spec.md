# Spec: Issue #5936 - Task: Wire Data Layer M0-M11 into service runtime paths with staged activation

- Issue: #5936
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5919

## Problem Statement
M0-M11 modules are mostly disconnected from Service API/runtime behavior.

## Scope
In scope:
- Define staged integration points and connect modules to real API/runtime workflows.

Out of scope:
- Net-new data-layer feature invention beyond existing module contracts.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: At least one production path per M0-M11 module is exercised through runtime/API integration.
- AC-2: Behavioral contracts and dependency boundaries for each module are documented.
- AC-3: Integration tests validate end-to-end usage of newly wired paths.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify At least one production path per M0-M11 module is exercised through runtime/API integration.
- C-02 (Functional, AC-2): Verify Behavioral contracts and dependency boundaries for each module are documented.
- C-03 (Functional, AC-3): Verify Integration tests validate end-to-end usage of newly wired paths.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: module-level behavior tests
- Functional: service-facing behavior paths
- Integration: API/runtime execution using M0-M11 components
- Regression: unwired-path checks fail when module disconnected
- Performance: critical path overhead monitored

## Dependencies
- #5919

