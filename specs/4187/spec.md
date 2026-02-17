# Spec — Issue #4187

- Title: update upgrade-governance docs and drift-contract tests for compatibility closure
- Parent: #4179
- Milestone: R27.21 Kolme cross-version upgrade compatibility governance
- Status: Reviewed
- Priority: P1

## Problem Statement

Upgrade compatibility smoke governance currently enforces CI strategy markers but does not fail closed
on drift between checker contracts and plan/ops closure evidence, creating a policy-doc mismatch risk.

## Scope

In scope:
- extend upgrade compatibility CI smoke checker to enforce production-plan marker parity,
- update production plan and ops planning docs with R27.21 closure markers,
- extend docs contract tests to fail closed on marker drift.

Out of scope:
- roadmap reprioritization,
- workflow topology redesign,
- additional runtime behavior changes.

## Acceptance Criteria

- AC-1: checker fails closed when production-plan closure markers drift.
- AC-2: strategy/checker marker taxonomy remains aligned after plan-doc parity extension.
- AC-3: production-plan and ops planning docs include deterministic R27.21 closure evidence markers.
- AC-4: docs contract tests fail closed on closure-marker mismatch.

## Conformance Cases

- C-01 (Functional): baseline checker passes with `status=pass`, `final_decision=GO`,
  `reason_codes_value=none` using strategy + plan docs. (AC-1, AC-2)
- C-02 (Regression): plan-doc marker drift fails checker with
  `production_plan_upgrade_compatibility_convergence_markers_missing`. (AC-1)
- C-03 (Docs contract): production-service next-steps contract enforces R27.21 closure section
  and markers. (AC-3, AC-4)
- C-04 (Docs contract): kolm-devnet-ops docs test enforces R27.21 closure markers and command.
  (AC-3, AC-4)

## Success Metrics / Signals

- checker CLI and docs reflect deterministic plan-doc parity contract,
- production and ops plans carry the same upgrade-compatibility closure governance markers,
- docs-contract tests fail closed on marker drift and pass on baseline.
