# Spec: Issue #5929 - Task: Harden SDK HTTP request construction against path/header injection

- Issue: #5929
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P0
- Area: sdk
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5918

## Problem Statement
User-controlled identifiers are currently interpolated directly into request paths/headers.

## Scope
In scope:
- Apply strict path-segment encoding and header-value validation (or safe client migration).

Out of scope:
- API endpoint shape changes unrelated to injection remediation.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: CRLF and delimiter injection payloads are rejected or safely encoded.
- AC-2: No direct raw interpolation of untrusted IDs into HTTP request line/headers remains.
- AC-3: Integration tests validate malformed input is fail-closed.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify CRLF and delimiter injection payloads are rejected or safely encoded.
- C-02 (Functional, AC-2): Verify No direct raw interpolation of untrusted IDs into HTTP request line/headers remains.
- C-03 (Functional, AC-3): Verify Integration tests validate malformed input is fail-closed.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: encoder/sanitizer behavior
- Functional: SDK request builder with malicious inputs
- Integration: real HTTP server validates no injected headers
- Regression: known injection payloads blocked
- Performance: request construction non-regression

## Dependencies
- #5918

