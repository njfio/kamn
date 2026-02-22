# Spec: #5661 Verification Captured-At Format Contract

- Issue: #5661
- Milestone: R64 E2E Verification Captured-At Format Contract
- Status: Implemented
- Priority: P1

## Problem Statement
Verify currently enforces `_verification.captured_at` marker presence but not value format. PRD section 8.3 models `captured_at` as a UTC timestamp (`2026-02-21T14:31:05Z`).

## Scope
### In Scope
- Enforce deterministic rejection when `_verification.captured_at` is present but malformed.
- Require RFC3339 UTC-Z compatible timestamp format for `_verification.captured_at`.
- Preserve existing marker-presence, hash-format, block-height-format, and finality-value checks.
- Emit deterministic diagnostics for captured-at format violations.

### Out of Scope
- Cross-artifact chronology checks.
- Time-source trust and NTP/clock synchronization.

## Acceptance Criteria
### AC-1 Invalid captured-at format rejection
Given `_verification.captured_at` is present but not a valid RFC3339 UTC-Z timestamp,
When verify command runs,
Then verification fails with deterministic captured-at format error.

### AC-2 Deterministic diagnostics
Given invalid captured-at format appears in an evidence artifact,
When verify command runs,
Then the error identifies `_verification.captured_at` format contract violation.

### AC-3 Valid captured-at compatibility
Given `_verification.captured_at` is valid RFC3339 UTC-Z and other required contracts hold,
When verify command runs,
Then verification report generation succeeds.

## Conformance Cases
- C-01 (AC-1, AC-2): verify rejects malformed/non-RFC3339-UTC `captured_at` values.
- C-02 (AC-3): verify accepts valid RFC3339 UTC-Z `captured_at` values with other required contracts.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with captured-at format conformance coverage.
- `cargo test -p kamn-e2e-harness` green with no regressions.
