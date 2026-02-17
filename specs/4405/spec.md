# Spec — #4405 Subtask: Red Tests for Invariant Violation Acceptance Gaps

Status: Reviewed
Priority: P1
Parent: #4401
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Invariant checker tests do not currently enforce red coverage for lane-violation acceptance drift and unstable invariant evidence/taxonomy outputs.

## Scope

In scope:
- Add deterministic failing tests in invariant policy checker contract tests.
- Cover mismatch cases where reports claim pass despite lane violations.
- Cover taxonomy/evidence marker drift rejection.

Out of scope:
- Implementing checker behavior changes (handled in #4406).

## Acceptance Criteria

AC-1: Tests fail when lane status violations are accepted as pass.

AC-2: Tests fail when deterministic taxonomy/evidence fields are missing or tampered.

AC-3: Regression tests preserve deterministic fail-closed coverage for invariant policy behavior.

## Conformance Cases

- C-01 (AC-1, Regression): `property_lane_status=fail` with `status=pass` is rejected.
- C-02 (AC-2, Regression): taxonomy-version mismatch is rejected deterministically.
- C-03 (AC-2, Regression): missing deterministic reason marker fields are rejected.
- C-04 (AC-3, Functional): existing artifact-key tamper coverage remains intact.
