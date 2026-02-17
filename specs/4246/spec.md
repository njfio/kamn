# Spec — #4246 Subtask: Replay-Integrity CI Smoke Checker

Status: Reviewed
Priority: P1
Parent: #4239
Milestone: R27.25 Persistent journal replay and checkpoint-integrity governance

## Problem Statement

Fast-gate requires a dedicated replay-integrity CI smoke checker that enforces fail-closed marker drift detection and heavy sqlite crash-recovery run-mode exclusion.

## Scope

In scope:
- Implement checker and contract test for sqlite crash-recovery replay-integrity CI smoke convergence.
- Enforce deterministic reason taxonomy output and JSON report schema.
- Enforce ci-fast-gate + ci-tools fast-mode run-mode exclusion.

Out of scope:
- Runtime crash-recovery lane behavior changes.
- New run-mode heavy execution in fast-gate.

## Acceptance Criteria

AC-1: Checker verifies required sqlite smoke command composition in fast mode.

AC-2: Checker fails closed on run-mode leakage in ci-tools fast mode and ci-fast-gate workflow.

AC-3: Checker emits deterministic reason taxonomy markers and stable JSON outputs.

## Conformance Cases

- C-01 (AC-1, Functional): baseline checker returns GO with `reason_codes_value=none`.
- C-02 (AC-1, Regression): missing required smoke command fails with deterministic composition reason.
- C-03 (AC-2, Regression): leaked run-mode command in fast mode fails with deterministic leakage reason.
- C-04 (AC-2, Regression): leaked run-mode command in workflow fails with deterministic exclusion reason.
- C-05 (AC-3, Regression): max-seconds overflow fails with deterministic budget reason.
