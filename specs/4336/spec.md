# Spec — #4336 Subtask: RED Tests for Runtime Extraction Parity Drift Across Module Boundaries

Status: Implemented
Priority: P1
Parent: #4329
Milestone: R27.31 Signal-safe daemon lifecycle, streaming observability, and runtime-decomposition governance

## Problem Statement

Without RED tests for module-boundary parity markers, runtime extraction drift can bypass governance checks.

## Scope

In scope:
- RED tests for runtime module-boundary parity markers in validation and policy outputs.
- RED tests for deterministic drift reason mapping behavior.

Out of scope:
- Implementing checker logic itself.

## Acceptance Criteria

AC-1: Tests fail pre-implementation when expected module-boundary parity markers are absent.

AC-2: Tests fail pre-implementation when boundary drift reason mapping markers are absent or unstable.

AC-3: Regression assertions preserve deterministic marker output normalization.

## Conformance Cases

- C-01 (AC-1, Functional): validation output must include module-boundary taxonomy and parity status markers.
- C-02 (AC-2, Regression): policy output must include module-boundary reason taxonomy markers and deterministic reason values.
- C-03 (AC-3, Regression): tampered boundary status fails policy with deterministic boundary drift reason marker.

## Success Signals

- New tests fail before implementation and pass after checker integration.
