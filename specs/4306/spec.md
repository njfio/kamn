# Spec — #4306 Subtask: CI Smoke Checker Composition for Transport-Observability-TLS Convergence

Status: Reviewed
Priority: P1
Parent: #4299
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

CI fast-gate needs a deterministic, low-cost composite checker that catches transport,
observability, and TLS contract drift without running local-heavy lanes.

## Scope

In scope:
- Composite checker with deterministic fail-closed reason mapping.
- CI fast-mode composition command checks and local-heavy leak detection.
- CI smoke budget threshold guard markers.

Out of scope:
- Running full local-heavy integration drills in ci-fast-gate.

## Acceptance Criteria

AC-1: Composite checker fails closed on transport/observability/TLS contract mismatch.

AC-2: Reason outputs deterministically identify failing contract domain(s).

AC-3: CI smoke budget threshold checks are enforced.

## Conformance Cases

- C-01 (AC-1, Functional): checker passes on repository baseline.
- C-02 (AC-1/AC-2, Regression): missing transport composition marker fails with transport reason code.
- C-03 (AC-1/AC-2, Regression): missing observability composition marker fails with observability reason code.
- C-04 (AC-1/AC-2, Regression): missing TLS composition marker fails with TLS reason code.
- C-05 (AC-3, Performance): budget overflow parameter fails with deterministic max-seconds reason code.
- C-06 (AC-1/AC-2, Regression): local-heavy command leakage in fast mode fails with deterministic leakage reason code.

## Success Signals

- Checker output is deterministic and domain-attributed.
- CI smoke path remains low-cost and bounded.
