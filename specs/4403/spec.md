# Spec — #4403 Task: Observability Emission Payload Schema + Fail-Closed Reason Mapping

Status: Reviewed
Priority: P1
Parent: #4400
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Runtime observability emission payload checks must fail closed on schema drift and missing required fields with deterministic reason mapping and normalized evidence markers.

## Scope

In scope:
- Runtime observability emission policy schema validation hardening.
- Deterministic reason mapping for missing-field/schema-drift failures.
- Normalized reason output marker(s) for policy evidence.
- Runtime observability schema docs updates.

Out of scope:
- External telemetry backend integration changes.
- Non-runtime observability domains.

## Acceptance Criteria

AC-1: Required runtime observability emission fields are validated and missing fields fail closed with deterministic reason markers.

AC-2: Schema drift and marker drift are rejected with deterministic reason mapping.

AC-3: Policy evidence outputs include normalized reason marker values across pass/fail paths.

AC-4: Observability schema docs reflect payload checker matrix and failure taxonomy.

## Conformance Cases

- C-01 (AC-1, Functional): missing required field in summary report is rejected with deterministic missing-field reason code.
- C-02 (AC-2, Regression): schema/taxonomy/marker drifts continue to fail closed with deterministic reasons.
- C-03 (AC-3, Integration): policy output emits normalized `reason_codes_value` marker for GO and NO-GO paths.
- C-04 (AC-4, Docs): docs include runtime observability endpoint payload field/taxonomy matrix.

