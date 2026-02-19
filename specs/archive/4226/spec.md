# Spec — #4226 Subtask: Deterministic Concurrency Budget Checker Outputs + Fail-Closed Reason Mapping

Status: Implemented
Priority: P1
Parent: #4221
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement
After red tests exist, checker outputs must encode deterministic concurrency budget outcomes and reason mapping under fail-closed conditions.

## Scope
In scope:
- Implement concurrency budget checker outputs (status/decision/taxonomy/version/csv).
- Implement deterministic fail-closed reason mapping for budget mismatches.
- Propagate outputs to lane/report surfaces.

Out of scope:
- CI topology changes beyond command composition updates.

## Acceptance Criteria

AC-1: Checker output includes deterministic concurrency-budget marker taxonomy.

AC-2: Fail-closed reason mapping is deterministic for queue/in-flight budget mismatches.

AC-3: Lane output propagates deterministic concurrency-budget markers.

## Conformance Cases

- C-01 (AC-1, Functional): baseline checker output includes expected taxonomy/version/csv markers.
- C-02 (AC-2, Regression): queue-budget mismatch maps to deterministic fail-closed reason.
- C-03 (AC-2, Regression): in-flight-budget mismatch maps to deterministic fail-closed reason.
- C-04 (AC-3, Integration): lane output includes propagated checker markers.
