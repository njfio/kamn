# Spec: Issue #4453

Status: Implemented
Issue: #4453
Parent: #4447
Milestone: R27.39 Runtime decomposition, panic-free execution, and dependency-license governance
Priority: P1

## Problem Statement

The generic test-harness LOC soft-budget checker and contract lane emit deterministic
`reason_codes`, but they do not yet publish a full normalized reason-taxonomy surface
(`reason_taxonomy_version`, `reason_codes_csv`, `reason_codes_value`, and reason-class
classification) for structural-budget automation. CI smoke enforcement is bounded by a
runtime threshold, but explicit low-cost boundary markers are not emitted as a stable
contract for docs/tests.

## Scope

In scope:
- Deterministic structural-budget reason taxonomy markers for
  `check_test_harness_loc_soft_budget.py` outputs.
- Deterministic CI smoke boundary/enforcement markers for
  `run_test_harness_loc_soft_budget_contract_lane.sh` outputs and report payload.
- Conformance and docs contracts for taxonomy and CI smoke marker surfaces.

Out of scope:
- Changing soft-budget/trend thresholds themselves.
- Expanding fast-gate to run heavy integration or local-only lanes.
- Workflow topology rewrites.

## Acceptance Criteria

AC-1:
Given any soft-budget checker result (within/exceeded/trend warn/trend fail/error), when
output is emitted, then deterministic reason-taxonomy markers are present:
`reason_taxonomy_version`, `reason_codes_csv`, `reason_codes_value`, and `reason_class`.

AC-2:
Given contract-lane execution for CI smoke structural-budget checks, when lane output and
summary report are emitted, then bounded CI-smoke enforcement markers are present and stable:
`ci_smoke_lane_cost_profile`, `ci_smoke_runtime_budget_status`, and deterministic reason key.

AC-3:
Given targeted script and docs contract tests, when they run, then taxonomy/CI-smoke markers
are validated fail-closed for checker, contract-lane, and CI strategy documentation.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `bash scripts/ci/test_check_test_harness_loc_soft_budget.sh`
  - Expectation: checker output emits deterministic taxonomy markers and normalized reason value
    across within/exceeded/error paths.

- C-02 (AC-2, Integration/Conformance):
  - Test: `bash scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh`
  - Expectation: contract-lane stdout/report include deterministic CI-smoke bounded enforcement
    markers and reason key contract.

- C-03 (AC-3, Regression/Conformance):
  - Test: `cargo test -p kamn-core --test ci_strategy_docs`
  - Expectation: `docs/ci/strategy.md` documents reason taxonomy and bounded CI-smoke markers.

## Success Metrics / Observable Signals

- Checker output gains deterministic machine-parse markers without breaking existing consumers.
- Contract lane remains low-cost and emits explicit bounded-enforcement status.
- Targeted shell and docs contract tests fail closed on taxonomy/CI-smoke drift.
