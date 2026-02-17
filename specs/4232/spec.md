# Spec — #4232 Subtask: Admission/Backpressure Docs + Drift-Contract Alignment

Status: Reviewed
Priority: P1
Parent: #4224
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement

Admission/backpressure checker behavior can drift from strategy/plan docs unless deterministic markers and boundary statements are pinned by docs-contract tests.

## Scope

In scope:
- Update `docs/ci/strategy.md` with admission/backpressure CI smoke governance markers.
- Update `docs/plans/2026-02-14-production-service-next-steps.md` with R27.24 closure chain and markers.
- Extend docs-contract tests to fail closed on marker drift.

Out of scope:
- Broad roadmap reprioritization.
- Runtime behavior changes outside docs/test contracts.

## Acceptance Criteria

AC-1: CI strategy docs include admission/backpressure checker commands and deterministic taxonomy markers.

AC-2: Production next-steps plan includes R27.24 closure chain and convergence markers.

AC-3: Docs-contract tests fail closed when required markers drift.

## Conformance Cases

- C-01 (AC-1, Docs): strategy doc includes checker command, taxonomy version, reason codes CSV, and boundary markers.
- C-02 (AC-2, Docs): plan doc includes R27.24 closure chain and convergence markers.
- C-03 (AC-3, Regression): docs-contract tests assert required markers and fail deterministically on drift.
