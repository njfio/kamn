# Spec — #4350 Subtask: Deterministic Allowlist Delta Evidence Outputs

Status: Reviewed
Priority: P1
Parent: #4343

## Problem Statement

The checker computes throughput/velocity policy but hides key evidence outputs, limiting auditability for allowlist graduation governance.

## Acceptance Criteria

AC-1: Checker emits deterministic markers for allowlisted/graduated counts and deltas.

AC-2: Checker emits deterministic velocity reason markers from policy evaluation in both pass/fail paths.

AC-3: Existing policy behavior and drift guards remain unchanged aside from additive evidence output.

## Conformance Cases

- C-01 (AC-1): pass output includes count/delta markers.
- C-02 (AC-2): stagnation failure output includes reason taxonomy/value markers.
- C-03 (AC-3): prior drift regression tests remain green.
