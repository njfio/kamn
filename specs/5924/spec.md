# Spec: Issue #5924 - Task: Replace kamn-core wipe_bytes loop with compiler-safe zeroization

- Issue: #5924
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: task
- Priority: P1
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Parent story: #5916

## Problem Statement
Manual byte loops may be optimized away and are not sufficient for secret erasure guarantees.

## Scope
In scope:
- Use zeroize or equivalent guaranteed erasure primitive in signature_profile key material paths.

Out of scope:
- Broad cryptographic API refactor unrelated to zeroization correctness.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: All sensitive buffers in kamn-core signing paths are zeroized using proven primitives.
- AC-2: Failure-path tests verify no secret leaks in error output.
- AC-3: Static checks prevent reintroduction of manual wipe loops in sensitive modules.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify All sensitive buffers in kamn-core signing paths are zeroized using proven primitives.
- C-02 (Functional, AC-2): Verify Failure-path tests verify no secret leaks in error output.
- C-03 (Functional, AC-3): Verify Static checks prevent reintroduction of manual wipe loops in sensitive modules.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: zeroization behavior in success/failure paths
- Functional: signing flows under error conditions
- Integration: signer profile end-to-end path
- Regression: manual wipe loop removal coverage
- Performance: N/A (documented)

## Dependencies
- #5916

