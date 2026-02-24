# Spec: Issue #5923 - Task: Replace deterministic agent-lib auth signatures with cryptographic signatures

- Issue: #5923
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P0
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5916

## Problem Statement
Auth currently provides zero authenticity because signature and verification are deterministic string recomputation.

## Scope
In scope:
- Implement cryptographic auth signatures and verification aligned with supported signature profile.

Out of scope:
- Agent-lib API redesign unrelated to auth correctness.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Forged deterministic signatures no longer validate.
- AC-2: Auth signing requires real private-key material and produces verifiable signatures.
- AC-3: Regression tests cover replay/tamper/forgery negative paths.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Forged deterministic signatures no longer validate.
- C-02 (Functional, AC-2): Verify Auth signing requires real private-key material and produces verifiable signatures.
- C-03 (Functional, AC-3): Verify Regression tests cover replay/tamper/forgery negative paths.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: auth sign/verify functions
- Functional: request auth round-trip
- Integration: sdk/agent-lib authenticated request path
- Regression: deterministic signature string path removed
- Performance: auth signing latency budget

## Dependencies
- #5916

