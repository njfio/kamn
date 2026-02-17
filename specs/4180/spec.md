# Spec — Issue #4180

- Title: add red tests for compatibility marker matrix mismatch rejection
- Parent: #4176
- Milestone: R27.21 Kolme cross-version upgrade compatibility governance
- Status: Implemented
- Priority: P1

## Problem Statement

Upgrade compatibility currently validates individual version/fork reports, but lacks explicit
red-fixture coverage that proves runtime/schema/failure-taxonomy marker matrix mismatches are
rejected deterministically.

## Scope

In scope:
- add failing mismatch fixtures for compatibility marker matrix validation,
- assert deterministic fail-closed reason outputs for mismatched markers.

Out of scope:
- new compatibility framework design,
- release workflow redesign.

## Acceptance Criteria

- AC-1: red fixtures fail when report schema marker drifts.
- AC-2: red fixtures fail when reason taxonomy/csv markers drift.
- AC-3: red fixtures fail when rehearsal guard markers drift.
- AC-4: mismatch outputs include deterministic fail-closed reason codes.

## Conformance Cases

- C-01: tampered version-report schema fails with `version_report_schema_mismatch`. (AC-1)
- C-02: tampered version-report reason taxonomy fails with
  `version_report_reason_taxonomy_mismatch`. (AC-2)
- C-03: tampered fork-policy reason CSV fails with
  `fork_policy_report_reason_codes_csv_mismatch`. (AC-2)
- C-04: tampered fork-policy rehearsal guard marker fails with
  `fork_policy_report_rehearsal_bypass_guard_status_mismatch`. (AC-3)
- C-05: fail output preserves deterministic reason-ordering taxonomy markers. (AC-4)

## Success Metrics / Signals

- red mismatch fixtures are executable in existing version-compatibility contract lane tests,
- each mismatch fixture produces deterministic fail-closed reason markers.
