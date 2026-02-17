# Spec — #4349 Subtask: RED Tests for Graduation Regression/Stagnation

Status: Reviewed
Priority: P1
Parent: #4343

## Problem Statement

Current checker tests do not enforce deterministic allowlist delta evidence outputs, so regressions can pass without auditable marker guarantees.

## Acceptance Criteria

AC-1: Tests assert deterministic missing-docs evidence markers on checker pass path.

AC-2: Tests assert deterministic missing-docs evidence markers on checker stagnation failure path.

AC-3: Existing regression assertions remain intact.

## Conformance Cases

- C-01 (AC-1): pass run includes expected `missing_docs_*` marker keys and non-empty values.
- C-02 (AC-2): stagnation failure includes expected `missing_docs_*` marker keys and reason fields.
- C-03 (AC-3): pre-existing failure scenarios still fail.
