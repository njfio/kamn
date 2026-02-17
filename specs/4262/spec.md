# Spec — #4262 Subtask: Docs and Drift-Contract Parity for Partition-Finality CI Smoke Governance

Status: Reviewed
Priority: P1
Parent: #4254
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

Checker/runtime/documentation marker drift can silently weaken CI governance if strategy and plan
artifacts are not contract-validated.

## Scope

In scope:
- Update strategy and production plan markers for partition-finality CI smoke governance.
- Update docs-contract tests to assert marker parity.

Out of scope:
- Roadmap reprioritization.

## Acceptance Criteria

AC-1: strategy doc includes checker command and deterministic taxonomy markers.

AC-2: production plan includes closure markers and checker command.

AC-3: docs-contract tests fail closed on drift.

## Conformance Cases

- C-01 (AC-1): `docs/ci/strategy.md` contains all required checker markers.
- C-02 (AC-2): `docs/plans/2026-02-14-production-service-next-steps.md` contains closure markers.
- C-03 (AC-3): docs-contract tests detect marker drift.
