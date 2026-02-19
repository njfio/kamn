# Spec: Issue #4342

Status: Implemented
Issue: #4342
Parent: #4339
Milestone: R27.32 Script-surface consolidation, documentation graduation, and architecture-navigability governance
Priority: P1

## Problem Statement

Script-surface trend checks currently emit pass/fail and reason codes, but deterministic reason
taxonomy markers and explicit CI smoke runtime budget enforcement are not consistently enforced
across non-Kolme wave trend gates.

## Scope

In scope:
- RED tests for missing taxonomy markers and unbounded runtime budget behavior.
- Deterministic reason taxonomy marker emission for wrapper trend checks.
- CI smoke runtime budget enforcement and deterministic over-budget reason mapping.
- Docs updates for taxonomy and budget marker contracts.

Out of scope:
- Heavy analytics or local-heavy execution in fast gate.

## Acceptance Criteria

AC-1:
Given non-Kolme wave wrapper trend checks, when checks run, then deterministic reason taxonomy
markers are emitted in stdout and JSON reports.

AC-2:
Given CI smoke budget policy, when runtime exceeds configured budget, then checks fail closed with
a deterministic runtime budget reason code.

AC-3:
Given trend-threshold regressions, when failures occur, then reason-code-to-decision mapping remains
deterministic and machine-readable.

AC-4:
Given docs parity checks, when taxonomy/budget marker contracts drift, then docs checks fail closed.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`
  - Expectation: taxonomy markers are emitted deterministically.

- C-02 (AC-2, Regression):
  - Test: `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`
  - Expectation: runtime budget exceed path fails closed with deterministic reason code.

- C-03 (AC-3, Integration):
  - Test: `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh`
  - Expectation: trend contract lane remains deterministic and bounded.

- C-04 (AC-4, Docs):
  - Test: `bash scripts/ci/test_ci_strategy_contract.sh`
  - Expectation: CI strategy contract includes updated taxonomy/budget markers.

## Success Metrics / Observable Signals

- Trend check outputs include stable taxonomy version and reason marker fields.
- Runtime budget overruns are deterministic and fail-closed in CI smoke paths.
