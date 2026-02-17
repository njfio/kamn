# Spec — #4269 Task: CI Smoke Governance for Websocket-Session Marker Integrity

Status: Implemented
Priority: P1
Parent: #4265
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Fast-gate requires deterministic websocket-session marker integrity checks without executing heavy session drills, otherwise governance drift can pass silently while CI cost grows unpredictably.

## Scope

In scope:
- CI smoke checker for websocket-session marker drift and heavy-lane exclusion enforcement.
- Deterministic fail-closed reason taxonomy for marker mismatch and boundary leakage.
- Docs + docs-contract updates for CI boundary and marker parity.

Out of scope:
- Always-on heavy session execution in fast-gate.
- CI topology redesign.

## Acceptance Criteria

AC-1: CI checker fails closed on websocket-session marker drift.

AC-2: Heavy websocket session drills remain excluded from fast-gate deterministically.

AC-3: Docs and docs-contract tests enforce marker and boundary parity.

AC-4: Focused Unit/Functional/Integration/Regression checks pass.

## Conformance Cases

- C-01 (AC-1, Functional): valid websocket-session marker composition passes checker.
- C-02 (AC-1, Regression): missing session marker command fails with deterministic reason code.
- C-03 (AC-2, Regression): leaked heavy session run command in fast mode fails with deterministic boundary reason.
- C-04 (AC-2, Regression): leaked heavy session workflow command fails deterministic exclusion guard.
- C-05 (AC-3, Conformance): docs-contract tests enforce strategy + next-steps marker parity.
- C-06 (AC-4, Integration): CI tools fast mode passes with checker wired and bounded runtime.

## Success Signals

- Checker reports stable, ordered reason codes and fail-closed decisions.
- Heavy-lane exclusion remains explicit and regression-enforced.
- Docs/checker marker contracts stay synchronized.
