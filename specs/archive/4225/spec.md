# Spec — #4225 Subtask: Red Tests for Async API Concurrency Budget Drift

Status: Implemented
Priority: P1
Parent: #4221
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement
Concurrency budget behavior requires deterministic failing tests before checker implementation to prevent aspirational merges.

## Scope
In scope:
- Add red tests for queue-limit rejection and in-flight budget mismatch paths.
- Verify deterministic fail-closed reason markers for tampered artifacts.

Out of scope:
- Final checker implementation details.

## Acceptance Criteria

AC-1: Queue-budget tamper fixtures fail deterministically.

AC-2: In-flight budget tamper fixtures fail deterministically.

AC-3: Regression tests preserve reason-marker stability/order.

## Conformance Cases

- C-01 (AC-1, Regression): queue-budget mismatch emits deterministic fail-closed marker.
- C-02 (AC-2, Regression): in-flight-budget mismatch emits deterministic fail-closed marker.
- C-03 (AC-3, Regression): repeated tamper runs preserve deterministic reason output.
