# Spec — #4256 Subtask: Deterministic Partition-Healing Checker Outputs and Fail-Closed Reason Mapping

Status: Implemented
Priority: P1
Parent: #4251
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

The partition-rejoin policy checker reports failed checks, but does not expose a deterministic mismatch-reason mapping contract suitable for governance/reporting parity.

## Acceptance Criteria

AC-1: Policy checker emits deterministic partition-healing mismatch mapping markers.

AC-2: Representative mismatch scenarios resolve to stable mismatch reason categories.

AC-3: Contract lane and docs/checklist surfaces enforce the mapping marker contract.

## Conformance Cases

- C-01 (AC-1): GO path emits mapping markers with `reason_code=none`.
- C-02 (AC-2): marker and reason-code payload mismatches resolve to deterministic mapping categories.
- C-03 (AC-3): contract lane checks and release checklist markers include deterministic mapping contract.
