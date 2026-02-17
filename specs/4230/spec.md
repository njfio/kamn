# Spec — #4230 Subtask: Overload Evidence Verifier + Promotion Reason Mapping

Status: Implemented
Priority: P1
Parent: #4223
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement

Promotion decisions need deterministic reason mapping tied to verified overload evidence convergence outcomes.

## Scope

In scope:
- Implement evidence verifier logic for axum lane/policy/source artifacts.
- Implement deterministic promotion decision reason mapping verification.
- Emit explicit reason taxonomy/version/csv markers in convergence outputs.

Out of scope:
- External release orchestration redesign.

## Acceptance Criteria

AC-1: Convergence verifier rejects invalid/missing/tampered evidence deterministically.

AC-2: Promotion reason mapping verification is deterministic and stable.

AC-3: Lane integration propagates convergence and reason-mapping markers.

## Conformance Cases

- C-01 (AC-1, Functional): baseline convergence checker returns GO and `reason_codes_value=none`.
- C-02 (AC-1, Regression): invalid link/payload reasons fail closed deterministically.
- C-03 (AC-2, Integration): observed reason mapping mismatch is rejected with deterministic reason.
- C-04 (AC-3, Regression): lane output includes convergence and promotion reason taxonomy markers.
