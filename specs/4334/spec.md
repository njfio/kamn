# Spec — #4334 Subtask: RED Tests for Observability Schema Drift

Status: Reviewed
Priority: P1
Parent: #4328
Milestone: R27.31 Signal-safe daemon lifecycle, streaming observability, and runtime-decomposition governance

## Problem Statement

Schema drift and missing required fields in observability endpoint payloads can pass unnoticed unless deterministic RED tests fail immediately.

## Scope

In scope:
- RED tests for missing required fields across endpoint payload surfaces.
- RED tests for schema-version drift classification and deterministic reason output.

Out of scope:
- Implementing checker logic itself.

## Acceptance Criteria

AC-1: Missing-field tests fail against pre-checker behavior.

AC-2: Schema-drift tests fail against pre-checker behavior.

AC-3: Test assertions encode deterministic reason taxonomy format for checker failures.

## Conformance Cases

- C-01 (AC-1, Unit): health payload missing `reason_code` maps to required-field failure reason.
- C-02 (AC-1, Unit): metrics payload missing readiness reason metric maps to required-field failure reason.
- C-03 (AC-2, Unit): stream payload schema version drift maps to schema-drift failure reason.
- C-04 (AC-3, Regression): reason taxonomy version marker and fail-closed status markers are asserted as stable.

## Success Signals

- New tests fail before checker implementation and pass after implementation.
