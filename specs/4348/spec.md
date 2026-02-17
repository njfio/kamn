# Spec: Issue #4348

Status: Implemented
Issue: #4348
Parent: #4342
Milestone: R27.32 Script-surface consolidation, documentation graduation, and architecture-navigability governance
Priority: P1

## Problem Statement

Budget checker output lacks explicit reason taxonomy markers and runtime-budget fail-closed mapping
for low-cost CI smoke governance.

## Scope

In scope:
- Implement deterministic reason taxonomy markers in trend checker output/report payloads.
- Implement runtime budget enforcement with deterministic overrun reason code.
- Thread runtime budget control through wave-19 trend wrapper.

Out of scope:
- Cross-repo policy rewrites outside wrapper trend checker lane.

## Acceptance Criteria

AC-1:
Given wrapper trend checks, when checks run, then `reason_taxonomy_version` and
`reason_codes_value` markers are emitted deterministically.

AC-2:
Given runtime budgets, when elapsed runtime exceeds budget, then policy fails with deterministic
reason code `ci_smoke_runtime_budget_exceeded`.

AC-3:
Given wrapper trend reports, when parsed, then policy decision and runtime budget status fields are
machine-readable and stable.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`

- C-02 (AC-2, Regression):
  - Test: `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`

- C-03 (AC-3, Integration):
  - Tests:
    - `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh`
    - `bash scripts/ci/test_ci_strategy_contract.sh`

## Success Metrics / Observable Signals

- Trend checker emits deterministic taxonomy/runtime markers in both stdout and JSON reports.
