# Spec — #4277 Subtask: Websocket-Session Governance Docs and Drift-Contract Sync

Status: Reviewed
Priority: P1
Parent: #4269
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Docs/checker drift can invalidate governance evidence even when checker logic is correct.

## Acceptance Criteria

AC-1: CI strategy includes websocket-session CI smoke convergence governance markers.

AC-2: Production next-steps plan includes R27.27 closure markers and chain evidence.

AC-3: Docs-contract tests fail closed when required websocket-session markers are removed.

## Conformance Cases

- C-01 (AC-1): strategy doc includes checker command, taxonomy, budget, and boundary markers.
- C-02 (AC-2): next-steps plan includes active chain and convergence marker set.
- C-03 (AC-3): shell and Rust docs-contract tests enforce marker presence.
