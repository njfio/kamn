# Spec: #4366 RED Tests for Deployment-Safety Evidence Gaps

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

Current milestone aggregate tests do not fail when rotation taxonomy evidence or CI/local-heavy boundary markers drift.

## Scope

In scope:
- Red fixtures/assertions for deployment preflight rotation taxonomy drift.
- Red fixtures/assertions for go/no-go boundary marker drift.

Out of scope:
- Checker implementation logic (handled in #4367).

## Acceptance Criteria

AC-1 Tests fail when deployment preflight rotation taxonomy markers are missing or mismatched.
AC-2 Tests fail when go/no-go boundary markers are mismatched.
AC-3 Regression path preserves deterministic mismatch reason codes.

## Conformance Cases

- C-01 (AC-1): rotation taxonomy version drift fixture yields fail-closed reason.
- C-02 (AC-2): boundary marker drift fixture yields fail-closed reason.
- C-03 (AC-3): deterministic reason code assertions hold across reruns.

## Success Metrics

- Added tests fail before checker updates and pass after implementation.
