# Spec — Issue #4202

- Title: update convergence-governance docs and drift-contract tests for live-validation closure
- Parent: #4194
- Milestone: R27.22 End-to-end live validation harness and promotion evidence convergence
- Status: Implemented
- Priority: P1

## Problem Statement

The local full-stack CI smoke convergence checker and governance documentation are partially
synchronized. Plan/ops closure markers are not yet enforced as deterministic drift contracts across
CI strategy, production plan, and Kolme devnet operations documentation.

## Scope

In scope:
- synchronize local full-stack CI smoke governance markers across:
  - `docs/ci/strategy.md`,
  - `docs/plans/2026-02-14-production-service-next-steps.md`,
  - `docs/planning/kolme-devnet-ops.md`.
- extend checker/test contracts to fail closed when production-plan marker declarations drift.
- add/update docs-contract tests that assert closure markers for plan and ops docs.

Out of scope:
- new lane orchestration behavior,
- changes to runtime live harness command implementation.

## Acceptance Criteria

- AC-1: Local full-stack CI smoke checker validates production-plan marker parity with deterministic
  fail-closed reason output.
- AC-2: Production plan doc includes explicit R27.22 CI smoke closure markers/commands/boundaries.
- AC-3: Kolme devnet ops doc includes aligned CI smoke closure markers and boundary declarations.
- AC-4: Drift-contract tests fail closed on marker taxonomy mismatch for docs/checker alignment.

## Conformance Cases

- C-01 (Functional): Baseline checker run passes with strategy + plan markers aligned. (AC-1)
- C-02 (Regression): Missing plan marker fixture fails checker with deterministic
  `production_plan_local_full_stack_convergence_markers_missing`. (AC-1, AC-4)
- C-03 (Docs Contract): Production-service next-steps contract requires R27.22 marker set and
  checker command. (AC-2, AC-4)
- C-04 (Docs Contract): Kolme devnet ops docs contract requires R27.22 CI smoke marker set and
  checker command. (AC-3, AC-4)

## Success Metrics / Signals

- Checker marker taxonomy and reason codes are consistent across CI strategy + plan + ops docs.
- Drift fixtures for plan-marker tamper are deterministic.
- Docs contracts for plan/ops remain fail-closed in CI.
