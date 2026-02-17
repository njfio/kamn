# Spec — #4247 Subtask: Replay-Governance Docs + Drift Contract Alignment

Status: Reviewed
Priority: P1
Parent: #4239
Milestone: R27.25 Persistent journal replay and checkpoint-integrity governance

## Problem Statement

Replay-governance checker contracts can drift from strategy/plan docs unless marker taxonomy and boundary statements are pinned by docs-contract tests.

## Scope

In scope:
- Update `docs/ci/strategy.md` with sqlite replay-integrity CI smoke governance markers.
- Update `docs/plans/2026-02-14-production-service-next-steps.md` with R27.25 closure chain and markers.
- Extend docs-contract tests to fail closed on marker drift.

Out of scope:
- Broader roadmap reprioritization.
- Runtime behavior changes outside docs/test contracts.

## Acceptance Criteria

AC-1: CI strategy docs include sqlite replay-integrity smoke checker commands and deterministic markers.

AC-2: Production next-steps plan includes R27.25 closure chain and marker surface.

AC-3: Docs-contract tests fail closed when required markers drift.

## Conformance Cases

- C-01 (AC-1, Docs): strategy doc includes checker command, reason taxonomy version, reason codes CSV, and boundary markers.
- C-02 (AC-2, Docs): plan doc includes R27.25 closure chain and convergence markers.
- C-03 (AC-3, Regression): docs-contract tests assert required markers and fail deterministically on drift.
