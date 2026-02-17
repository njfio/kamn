# Spec — #4255 Subtask: Red Tests for Partition Marker Mismatch and Deterministic Healing Rejection

Status: Reviewed
Priority: P1
Parent: #4251
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

Policy tests cover several tamper paths, but missing-marker and nondeterministic mismatch payload rejection needs explicit regression coverage.

## Acceptance Criteria

AC-1: Tests fail on missing partition/healing markers.

AC-2: Tests fail on nondeterministic mismatch payloads (e.g., unsorted/duplicated reason-code arrays).

AC-3: Repeated mismatch runs preserve deterministic fail-closed reason projection.

## Conformance Cases

- C-01 (AC-1): missing partition/healing marker fails policy check with deterministic reason output.
- C-02 (AC-2): unsorted/duplicated reconciliation reason-code payload fails policy check deterministically.
- C-03 (AC-3): repeated mismatch invocation keeps stable failed-check and mapped-reason outputs.
