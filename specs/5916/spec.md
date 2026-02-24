# Spec: Issue #5916 - Story: Cryptography and Auth Integrity Remediation

- Issue: #5916
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: story
- Priority: P0
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent epic: #5915

## Problem Statement
Production code paths still include deterministic/fake cryptography and non-cryptographic auth/signature constructs.

## Scope
In scope:
- Replace fake primitives with production cryptography and secure key-handling semantics.

Out of scope:
- Unrelated feature additions.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: All production messaging/auth/signature/hash paths use cryptographic primitives only.
- AC-2: Key material handling and zeroization are fail-closed and compiler-safe.
- AC-3: Regression tests prove no deterministic signature/crypto fallback remains in production paths.

## Conformance Cases
- C-01 (Functional, AC-1): Verify All production messaging/auth/signature/hash paths use cryptographic primitives only.
- C-02 (Functional, AC-2): Verify Key material handling and zeroization are fail-closed and compiler-safe.
- C-03 (Functional, AC-3): Verify Regression tests prove no deterministic signature/crypto fallback remains in production paths.

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

