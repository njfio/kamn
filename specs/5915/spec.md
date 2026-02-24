# Spec: Issue #5915 - Epic: R65 Security Runtime Remediation and Production Readiness

- Issue: #5915
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-24)
- Type: epic
- Priority: P0
- Area: program
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-24
- Parent: Program epic: #1

## Problem Statement
Critical security, cryptography, runtime-delivery, and test-assurance gaps currently block production deployment.

## Scope
In scope:
- Replace non-production crypto/signature/hash primitives in production paths.
- Deliver real end-to-end message delivery with durable persistence and bounded replay protection.
- Harden SDK/service transport and managed-signer execution surfaces.
- Expand integration/fuzz/mutation/coverage quality gates.
- Reduce architecture/governance drag (core decomposition, script-surface and duplication reduction).

Out of scope:
- New product features unrelated to production-readiness remediation.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: All listed critical findings are mapped to concrete child tasks with testable ACs.
- AC-2: Every child task is labeled, milestone-scoped, and parent-linked.
- AC-3: Production-readiness blockers are executable in dependency order with clear verification gates.

## Conformance Cases
- C-01 (Functional, AC-1): Verify All listed critical findings are mapped to concrete child tasks with testable ACs.
- C-02 (Functional, AC-2): Verify Every child task is labeled, milestone-scoped, and parent-linked.
- C-03 (Functional, AC-3): Verify Production-readiness blockers are executable in dependency order with clear verification gates.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: Required for each changed module
- Functional: Required for each behavior change
- Integration: Required for runtime/API/transport paths
- Regression: Required for each remediated finding
- Performance: Required for runtime/networking hot paths

## Dependencies
- None

