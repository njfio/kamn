# Spec — #4291 Subtask: CI Smoke Checker for Drift-Failover Marker Integrity

Status: Implemented
Priority: P1
Parent: #4284
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

Failover marker drift and fast-gate heavy-lane scope violations need deterministic CI smoke enforcement.

## Scope

In scope:
- CI smoke checker updates for failover marker drift.
- Heavy failover-lane exclusion checks in fast-gate scope.
- RED/regression coverage for deterministic drift/boundary reasons.

Out of scope:
- Running heavy failover lanes in fast-gate.

## Acceptance Criteria

AC-1: Checker fails closed on failover marker drift.

AC-2: Checker fails closed when heavy failover lane scope leaks into fast-gate.

AC-3: Drift/boundary mismatch reasons remain deterministic across repeated runs.

## Conformance Cases

- C-01 (AC-1, Functional): valid marker payload passes checker.
- C-02 (AC-1, Regression): drifted marker payload fails with deterministic reason.
- C-03 (AC-2, Regression): heavy-lane scope mismatch fails with deterministic reason.
- C-04 (AC-3, Regression): repeated mismatch checks preserve deterministic reason ordering.

## Success Signals

- CI smoke checker emits stable fail-closed reason markers for drift and boundary violations.
