# Spec: Issue #5930 - Task: Implement HTTPS support in SDK service client and TLS validation

- Issue: #5930
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P1
- Area: sdk
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5918

## Problem Statement
HTTPS is currently explicitly unsupported, forcing plaintext traffic.

## Scope
In scope:
- Implement HTTPS scheme support with certificate validation and clear configuration controls.

Out of scope:
- Mutual-TLS feature expansion not required for this task.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: SDK supports HTTPS endpoints with strict cert validation.
- AC-2: TLS misconfiguration and invalid cert chains fail closed with deterministic errors.
- AC-3: Integration tests with fixture certs pass.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify SDK supports HTTPS endpoints with strict cert validation.
- C-02 (Functional, AC-2): Verify TLS misconfiguration and invalid cert chains fail closed with deterministic errors.
- C-03 (Functional, AC-3): Verify Integration tests with fixture certs pass.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: scheme/config validation
- Functional: HTTPS request path
- Integration: HTTPS local fixture server
- Regression: NotImplemented HTTPS path removed
- Performance: connection overhead baseline documented

## Dependencies
- #5918

