# Spec: #4364 RED Tests for Key-Policy Violations and Stale Rotation Artifacts

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

Current tests validate fail-closed reasons but do not require deterministic key-policy/rotation taxonomy metadata outputs.

## Scope

In scope:
- Add RED assertions for missing taxonomy markers in GO output.
- Add NO-GO mapping assertions for stale rotation and key-source violations.

Out of scope:
- Checker logic changes (implemented in #4365).

## Acceptance Criteria

AC-1 GO report must require rotation preflight taxonomy markers.
AC-2 Stale rotation and key-source mismatch failures must require mapped taxonomy observed value.

## Conformance Cases

- C-01 (AC-1): missing taxonomy markers fail GO-path contract test.
- C-02 (AC-2): rotation-stalled failure maps into observed taxonomy value.
- C-03 (AC-2): production key-source mismatch failure maps into observed taxonomy value.

## Success Metrics

- Red tests fail before implementation and pass after implementation.
