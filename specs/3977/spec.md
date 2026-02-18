# Spec — Issue #3977

- Title: Subtask: add rustdoc smoke lane and low-cost CI contract checks for documentation build integrity
- Parent: #3969
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

Rustdoc lane governance exists but runtime-budget failure signaling is not explicit in the policy checker output surface, which weakens deterministic automation and docs-marker parity for smoke-lane budget behavior.

## Objective

Add deterministic runtime-budget markers and failure reason-code coverage to the kamn-core rustdoc artifact policy contracts, then align docs and CI contract assertions.

## Scope

In scope:
- Extend rustdoc artifact policy checker output with explicit runtime budget status marker.
- Add deterministic runtime-budget failure reason code in policy checker output.
- Expand rustdoc policy tests to cover runtime-budget failure path and success marker.
- Update docs marker contracts for new reason taxonomy/reason-csv surface.

Out of scope:
- External docs hosting/publishing workflow.
- New rustdoc lane families beyond kamn-core.

## Acceptance Criteria

- AC-1: Rustdoc artifact policy checker emits deterministic `runtime_budget_status=within|exceeded` marker.
- AC-2: Runtime-budget violations emit deterministic reason code surface for CI automation.
- AC-3: CI contract tests cover runtime-budget pass/fail marker paths.
- AC-4: Documentation marker contracts stay in sync with checker reason taxonomy/reason CSV.

## Conformance Cases

- C-01 (AC-1): Valid pass report emits `runtime_budget_status=within` from policy checker.
- C-02 (AC-2): Report with `runtime_seconds > max_runtime_seconds` fails with deterministic runtime-budget reason code.
- C-03 (AC-3): `test_check_kamn_core_rustdoc_artifact_policy.sh` enforces both C-01 and C-02.
- C-04 (AC-3): `test_run_kamn_core_rustdoc_artifact_contract_lane.sh` validates policy pass marker includes runtime budget status.
- C-05 (AC-4): `test_ci_strategy_contract.sh` validates updated rustdoc governance markers in docs.

## Success Metrics

- Rustdoc policy outputs deterministic runtime budget marker and reason taxonomy for failure automation.
- Rustdoc artifact policy/lane/tests remain green in fast CI tools regression.
