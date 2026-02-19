# Issue #4380 Spec

Status: Implemented
Priority: P1
Parent: #4373

## Problem Statement

Release/promotion review needs deterministic native signer reason mapping and stable signer evidence outputs in signed-to-Kolme policy reports.

## Scope

In scope:
- Native signer reason taxonomy constants and output fields in signed-to-Kolme policy checker.
- Runtime signing profile evidence normalization in signed-to-Kolme contract-lane summary.

Out of scope:
- New key management backends.

## Acceptance Criteria

AC-1:
- Native signer policy failures emit deterministic reason codes.

AC-2:
- Policy report outputs deterministic native signer taxonomy version/csv/value fields.

AC-3:
- Runtime signing profile evidence in signed-to-Kolme summary is stable and checker-validated.

## Conformance Cases

C-01 (AC-1, Functional): simulated signing marker present -> deterministic native signer reason code emitted.

C-02 (AC-1, Functional): native profile marker missing -> deterministic native signer reason code emitted.

C-03 (AC-2, Conformance): policy report contains deterministic taxonomy fields/values.

C-04 (AC-3, Integration): contract-lane run summary includes runtime signing profile evidence and passes checker.
