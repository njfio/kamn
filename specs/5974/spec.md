# Spec: Issue #5974 - Story: Service Transport Cryptographic Auth Upgrade

- Issue: #5974
- Status: Reviewed (agent-authored; explicit proceed directive on 2026-02-25)
- Type: story
- Priority: P1
- Area: security
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5973

## Problem Statement
Production service transport still includes deterministic baseline signature usage in request construction. Transport auth should be cryptographic for end-to-end assurance consistency.

## Scope
In scope:
- Replace deterministic production signature construction with cryptographic signing/verification.
- Keep explicit compatibility behavior for non-production/test contexts only.

Out of scope:
- Kolme chain signing path modifications.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: SDK/agent-lib production request signatures are cryptographic and service-verifiable.
- AC-2: Deterministic baseline signatures are blocked in production paths unless explicit test gate enabled.
- AC-3: Tamper/replay/wrong-key integration regressions are covered and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Cryptographic signature round-trip succeeds for service request path.
- C-02 (Regression, AC-2): Baseline deterministic signature path is rejected by default in production path.
- C-03 (Integration, AC-3): Tampered/replayed/wrong-key requests fail with deterministic reason codes.

## Success Metrics / Observable Signals
- No production code path generates baseline deterministic signatures.
- Service auth integration tests pass for positive and negative cryptographic cases.
