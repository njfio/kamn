# Spec — #4254 Task: Partition-Finality CI Smoke Governance and Heavy-Lane Exclusion

Status: Reviewed
Priority: P1
Parent: #4250
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

Partition-finality governance markers exist, but CI fast-gate still needs a dedicated composite
smoke checker that fail-closes on marker drift while guaranteeing heavy fault lanes remain
excluded from fast mode.

## Scope

In scope:
- Add composite CI smoke checker for partition-finality marker parity and fast-mode composition.
- Enforce deterministic heavy-lane exclusion checks for workflow + ci-tools fast mode.
- Add red/regression tests for drift, leakage, and budget-bound violations.
- Update strategy/plan docs and docs-contract tests for checker marker parity.

Out of scope:
- Executing heavy partition run-mode lanes in CI fast-gate.
- Runtime transport algorithm changes.

## Acceptance Criteria

AC-1: Composite checker fails closed on partition-finality marker drift.

AC-2: Heavy partition/finality run-mode lanes remain excluded from fast-gate and ci-tools fast mode.

AC-3: Strategy and production-plan docs stay synchronized with checker taxonomy markers.

AC-4: Checker runtime remains bounded for low-cost CI smoke use.

## Conformance Cases

- C-01 (AC-1, Functional): baseline checker run returns `status=pass` and deterministic marker set.
- C-02 (AC-1, Regression): docs/marker drift fixture returns deterministic taxonomy reason.
- C-03 (AC-2, Regression): leaked heavy run command in workflow or fast-mode block returns deterministic exclusion reason.
- C-04 (AC-4, Regression): max-seconds overflow returns deterministic budget reason.
- C-05 (AC-3, Integration): docs-contract tests enforce strategy/plan marker parity.

## Success Signals

- New checker report includes deterministic taxonomy version/codes and bounded smoke profile markers.
- Fast-mode CI runs checker test and fails closed on leakage or drift.
- Docs/tests converge on one marker taxonomy for partition-finality CI smoke governance.
