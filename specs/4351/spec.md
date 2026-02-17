# Spec — #4351 Subtask: RED Tests for Publication Drift + Ratio Imbalance

Status: Reviewed
Priority: P1
Parent: #4344

## Problem Statement

Current rustdoc artifact policy tests validate artifact integrity but do not enforce docs-heavy ratio imbalance fail-closed behavior.

## Acceptance Criteria

AC-1: Test suite fails when docs/behavioral ratio exceeds configured threshold.

AC-2: Failure emits deterministic reason marker for ratio imbalance.

AC-3: Existing artifact integrity regression checks remain green.

## Conformance Cases

- C-01 (AC-1): synthetic report with docs-heavy ratio fails checker.
- C-02 (AC-2): failure output includes deterministic ratio reason marker.
- C-03 (AC-3): existing checksum mismatch failure and happy path still validated.
