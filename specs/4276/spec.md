# Spec — #4276 Subtask: Websocket-Session CI Smoke Checker and Heavy-Lane Exclusion

Status: Reviewed
Priority: P1
Parent: #4269
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Without a deterministic checker, websocket-session marker drift and heavy-lane leakage can bypass fast-gate governance.

## Acceptance Criteria

AC-1: Checker fails closed when websocket-session smoke composition drifts.

AC-2: Checker fails closed when heavy websocket-session run commands appear in fast-gate paths.

AC-3: Checker enforces CI smoke runtime budget boundary and deterministic reason output.

## Conformance Cases

- C-01 (AC-1): baseline repository passes checker with `final_decision=GO`.
- C-02 (AC-1): missing websocket-session smoke command fixture fails with deterministic reason code.
- C-03 (AC-2): leaked local-heavy run command in fast-mode fixture fails deterministic leakage reason.
- C-04 (AC-2): leaked local-heavy workflow command fixture fails deterministic exclusion reason.
- C-05 (AC-3): max-seconds overflow fixture fails deterministic budget reason.
