# Spec — #4292 Subtask: Docs and Drift-Contract Parity for Failover CI Governance

Status: Implemented
Priority: P1
Parent: #4284
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

Docs and checker contracts can drift, weakening the failover CI governance boundary and closure evidence.

## Scope

In scope:
- Update CI strategy and next-steps docs for failover drift smoke markers and heavy-lane exclusion boundaries.
- Update docs-contract tests to fail closed on marker drift.

Out of scope:
- Roadmap reprioritization.

## Acceptance Criteria

AC-1: Docs include deterministic failover CI smoke marker and boundary contracts.

AC-2: Docs-contract tests fail closed on marker/boundary mismatch.

AC-3: Closure evidence references low-cost CI and local/scheduled heavy-lane boundaries.

## Conformance Cases

- C-01 (AC-1, Conformance): CI strategy doc contains failover drift checker and boundary markers.
- C-02 (AC-1, Conformance): next-steps doc contains R27.28 closure markers.
- C-03 (AC-2, Regression): docs-contract tests fail on removed/modified required markers.
- C-04 (AC-3, Integration): docs + checker references align with CI smoke-only scope.

## Success Signals

- Docs and checker contracts remain synchronized and test-enforced.
