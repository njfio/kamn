# Spec: Issue #5911 - Disable Legacy Baseline-v1 Signature Compatibility in Production Builds

- Issue: #5911
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Legacy deterministic baseline-v1 signature compatibility can currently be enabled with `KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1`. In non-debug production builds this creates an environment-controlled non-cryptographic signature acceptance path.

## Scope
In scope:
- Make legacy baseline-v1 compatibility helper fail closed in non-debug builds.
- Keep legacy compatibility behavior available only in debug/test workflows.
- Add regression tests for helper semantics and verify signing/transaction paths stay green.

Out of scope:
- Removing baseline-v1 fixture generation APIs.
- Refactoring signature-profile fixtures or docs beyond required behavior clarifications.

## Acceptance Criteria
### AC-1 Production compatibility path is disabled
Given a non-debug build,
When `KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1` is set,
Then legacy baseline-v1 compatibility remains disabled.

### AC-2 Debug/test compatibility remains explicit and deterministic
Given a debug/test build,
When `KAMN_SIGNER_ALLOW_LEGACY_BASELINE_V1` is set to a truthy value,
Then legacy baseline-v1 compatibility remains available for compatibility contracts.

### AC-3 Existing verification paths remain stable
Given current signer backend and transaction verification tests,
When compatibility helper is hardened,
Then no regression occurs in cryptographic verification behavior.

## Conformance Cases
- C-01 (AC-1, Unit): helper returns `false` in non-debug policy branch.
- C-02 (AC-2, Unit): helper remains env-gated for debug/test branch.
- C-03 (AC-3, Regression): signer backend compatibility tests and transaction guard compatibility tests remain green.

## Success Metrics / Observable Signals
- Legacy baseline-v1 acceptance cannot be env-enabled in production/release binaries.
- Debug/test compatibility fixtures continue to execute under explicit env control.
- No regression in signer backend / transaction contract lanes.
