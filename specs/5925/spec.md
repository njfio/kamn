# Spec: Issue #5925 - Task: Replace FNV-based key lifecycle audit chain hashing with cryptographic hash

- Issue: #5925
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P1
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5916

## Problem Statement
FNV-based hashing is collision-prone and weak for audit integrity claims.

## Scope
In scope:
- Replace audit-chain hash with SHA-256/BLAKE3 and version marker migration.

Out of scope:
- Audit UI/reporting feature expansion unrelated to hash integrity.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Audit chain records use cryptographic hash with explicit version marker.
- AC-2: Collision-oriented adversarial tests fail to violate chain integrity assumptions.
- AC-3: Backward-compatibility migration path is documented and tested.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Audit chain records use cryptographic hash with explicit version marker.
- C-02 (Functional, AC-2): Verify Collision-oriented adversarial tests fail to violate chain integrity assumptions.
- C-03 (Functional, AC-3): Verify Backward-compatibility migration path is documented and tested.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: hash-chain computation and versioning
- Functional: audit chain append/verify
- Integration: key lifecycle audit persistence and retrieval
- Regression: FNV path removed from production audit integrity checks
- Performance: chain update overhead non-regression

## Dependencies
- #5916

