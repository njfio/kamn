# Spec — #4409 Subtask: RED Tests for Emission Schema Drift + Missing Required Fields

Status: Reviewed
Priority: P1
Parent: #4403
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Runtime observability policy tests do not currently enforce deterministic fail-closed behavior for missing required payload fields.

## Scope

In scope:
- Add RED tests for missing required fields in runtime observability endpoint summary reports.
- Add RED checks for normalized reason marker output expectations.

Out of scope:
- Policy checker implementation changes (handled in #4410).

## Acceptance Criteria

AC-1: Tests fail when required summary report fields are missing.

AC-2: Tests fail when normalized reason output marker is absent.

AC-3: Existing schema/taxonomy drift regression checks remain intact.

## Conformance Cases

- C-01 (AC-1, Functional): missing `observability_tls_negative_matrix_reason_codes_csv` fails with deterministic reason marker.
- C-02 (AC-2, Functional): policy success/failure output must include deterministic `reason_codes_value`.
- C-03 (AC-3, Regression): existing tamper checks continue to validate deterministic fail-closed reasons.

