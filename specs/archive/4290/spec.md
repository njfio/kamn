# Spec — #4290 Subtask: Implement Convergence Verifier and Deterministic Promotion Reason Mapping

Status: Implemented
Priority: P1
Parent: #4283
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

Failover promotion decisions need deterministic, auditable reason mapping tied to converged drift evidence artifacts.

## Scope

In scope:
- Convergence verifier implementation for drift summary + policy artifacts.
- Deterministic promotion decision reason mapping fields in policy output.
- Fail-closed deterministic reason codes for link mismatch and payload drift.

Out of scope:
- Promotion orchestration engine redesign.

## Acceptance Criteria

AC-1: Verifier rejects invalid convergence artifacts deterministically.

AC-2: Promotion reason mapping is deterministic for both GO and NO-GO outcomes.

AC-3: Integration tests validate convergence-to-promotion decision linkage.

## Conformance Cases

- C-01 (AC-1, Functional): valid summary/policy artifacts verify convergence.
- C-02 (AC-1, Regression): missing linkage marker fails with deterministic reason.
- C-03 (AC-1, Regression): tampered payload fails with deterministic reason.
- C-04 (AC-2, Regression): promotion reason mapping mismatch fails with deterministic marker.
- C-05 (AC-3, Integration): suite contracts retain preflight pass markers with convergence signals.

## Success Signals

- Deterministic convergence verification and reason mapping are emitted in policy/checker outputs.
- Integration and regression tests pass with stable reason ordering.
