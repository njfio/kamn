# Spec — #4195 Subtask: Red Tests for Full-Stack Harness Marker Completeness and Parity Mismatch Rejection

Status: Implemented
Priority: P1
Parent: #4191
Milestone: R27.22 End-to-end live validation harness and promotion evidence convergence

## Problem Statement
Full-stack harness policy validation requires explicit failing regression tests for missing marker and parity mismatch paths so marker drift cannot pass silently.

## Scope
In scope:
- Add deterministic failing regression checks for missing harness marker rejection.
- Add deterministic failing regression checks for dry-run parity mismatch rejection.
- Document full-stack harness mismatch controls in ops configuration docs.
- Add docs-contract coverage for new ops markers.

Out of scope:
- Runtime/transport feature behavior changes outside policy validation.
- CI topology changes.

## Acceptance Criteria
AC-1 (Given/When/Then):
- Given a full I/O scenario matrix report with missing harness marker fields,
- When policy validation runs,
- Then validation fails closed with deterministic marker mismatch reason codes.

AC-2 (Given/When/Then):
- Given a dry-run report with command count/status parity drift,
- When policy validation runs,
- Then validation fails closed with deterministic parity mismatch reason codes.

AC-3 (Given/When/Then):
- Given operator configuration documentation for full-stack harness controls,
- When docs-contract tests run,
- Then required mismatch markers and regression references are present.

## Conformance Cases
- C-01 (AC-1, Regression): remove `process_harness_contract_status` and assert deterministic rejection reason.
- C-02 (AC-2, Regression): tamper `run_mode_command_count` in dry-run payload and assert deterministic rejection reason.
- C-03 (AC-2, Regression): tamper `run_mode_command_status` in dry-run payload and assert deterministic rejection reason.
- C-04 (AC-3, Docs): docs include full-stack harness mismatch controls and deterministic reason markers.

## Success Metrics / Signals
- Updated full I/O policy shell tests pass and include missing-marker/parity tamper assertions.
- Docs-contract tests pass with required full-stack harness controls.
