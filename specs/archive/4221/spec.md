# Spec — #4221 Task: Async API Concurrency Budget Checker + Deterministic Queue Markers

Status: Implemented
Priority: P1
Parent: #4219
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement
Async API concurrency budgets need deterministic enforcement and fail-closed evidence markers for in-flight and queue-boundary behavior.

## Scope
In scope:
- Enforce deterministic in-flight and queue budget checks for async API live contract lane artifacts.
- Emit deterministic reason taxonomy markers for budget mismatch and queue-limit rejection paths.
- Integrate checks into lane outputs, CI smoke composition, and docs contracts.

Out of scope:
- Queue architecture redesign.
- New API feature surface expansion.

## Acceptance Criteria

AC-1: Concurrency checker enforces configured in-flight and queue budgets deterministically.

AC-2: Budget violations fail closed with stable reason-code outputs.

AC-3: Lane/report outputs propagate deterministic concurrency budget markers for machine consumption.

AC-4: CI/docs contracts include concurrency-budget command surface and deterministic markers.

## Conformance Cases

- C-01 (AC-1, Functional): baseline artifacts return `GO` with deterministic concurrency marker status.
- C-02 (AC-2, Regression): queue budget tamper fails with deterministic fail-closed reason.
- C-03 (AC-2, Regression): in-flight budget tamper fails with deterministic fail-closed reason.
- C-04 (AC-3, Integration): lane output includes deterministic concurrency budget taxonomy/version/csv markers.
- C-05 (AC-4, Docs): docs-contract tests enforce command + marker references.
