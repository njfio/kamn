# Spec — #4284 Task: CI Smoke Governance for Drift-Failover Marker Integrity

Status: Implemented
Priority: P1
Parent: #4280
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

Fast-gate needs deterministic failover marker integrity checks without executing heavy failover lanes, otherwise CI cost and runtime become unstable and governance drift can pass unnoticed.

## Scope

In scope:
- CI smoke checker for failover marker drift and heavy-lane exclusion enforcement.
- Deterministic fail-closed reason taxonomy for marker drift and lane-boundary violations.
- Docs + docs-contract updates for CI boundary and marker parity.

Out of scope:
- Always-on heavy failover execution in fast-gate.
- CI topology redesign.

## Acceptance Criteria

AC-1: CI checker fails closed on drift-failover marker drift.

AC-2: Heavy failover lane execution remains excluded from fast-gate deterministically.

AC-3: Docs and docs-contract tests enforce marker/boundary parity.

AC-4: Focused Unit/Functional/Integration/Regression checks pass.

## Conformance Cases

- C-01 (AC-1, Functional): valid CI marker payload passes checker.
- C-02 (AC-1, Regression): drifted failover marker payload fails with deterministic drift reason.
- C-03 (AC-2, Regression): heavy-lane execution marker in fast-gate fails with deterministic boundary reason.
- C-04 (AC-2, Integration): CI smoke path preserves bounded fast-gate execution behavior.
- C-05 (AC-3, Conformance): docs-contract tests enforce CI strategy + next-steps marker parity.
- C-06 (AC-4, Regression): repeated mismatch checks preserve deterministic reason ordering.

## Success Signals

- CI checker detects failover drift deterministically with stable reason markers.
- Heavy-lane exclusion remains explicit and test-enforced.
- Docs/checker marker contracts remain synchronized.
