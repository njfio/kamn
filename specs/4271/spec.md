# Spec — #4271 Subtask: Deterministic Protocol Checker Outputs and Fail-Closed Reason Mapping

Status: Implemented
Priority: P1
Parent: #4266
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Policy checker mismatch outcomes need deterministic category mapping to keep protocol governance and promotion/runbook actions stable across runs.

## Acceptance Criteria

AC-1: Policy checker emits deterministic mismatch reason mapping markers.

AC-2: Mismatch scenarios project deterministic mapping reasons.

AC-3: Contract-lane integration enforces mapping marker presence.

## Conformance Cases

- C-01 (AC-1): successful checker output emits mapping markers with `reason_code=none`.
- C-02 (AC-2): representative mismatch cases map to stable category reasons.
- C-03 (AC-3): lane wrapper validates mapping markers from policy output.
- C-04 (AC-3): release checklist/docs-contract include deterministic mapping marker set.
