# Spec: Issue #5931 - Task: Harden managed signer execution and secret env handling

- Issue: #5931
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P1
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5918

## Problem Statement
Managed signer path uses sh -c and inherits full environment, including sensitive values.

## Scope
In scope:
- Switch to argv-safe process execution and explicit env allowlist for child process.

Out of scope:
- New managed signer backend capability beyond hardening scope.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: No managed-signer execution path uses shell command interpolation.
- AC-2: Child process environment excludes signer private key envs by default.
- AC-3: Security tests prove command-injection payloads and env leakage attempts fail.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify No managed-signer execution path uses shell command interpolation.
- C-02 (Functional, AC-2): Verify Child process environment excludes signer private key envs by default.
- C-03 (Functional, AC-3): Verify Security tests prove command-injection payloads and env leakage attempts fail.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: command builder and env scrubber
- Functional: managed signer invocation success/failure
- Integration: signer backend with controlled child process
- Regression: sh -c path removed
- Performance: signing invocation overhead bounded

## Dependencies
- #5918

