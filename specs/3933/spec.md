# Issue #3933 Spec

- Title: Epic: R27.5 remove panic paths and modularize node test surface for reliability closure
- Status: Implemented
- Type: epic
- Priority: P0
- Milestone: `specs/milestones/r27-5-panic-path-elimination-and-test-surface-modularization/index.md`

## Problem Statement
Reliability closure required both panic-path elimination and node test-surface modularization with fail-closed governance.

## Acceptance Criteria
- AC-1: panic-capable production paths replaced with typed fail-closed handling.
- AC-2: node test monoliths decomposed into focused files with behavior parity.
- AC-3: CI/docs governance checks fail closed on panic-policy or test-surface drift.

## Scope
In scope:
- Story `#3934` panic-path elimination (closed via #5159 lineage).
- Story `#3935` node test-surface decomposition + governance (closed via #5160/#5161/#5162/#5163/#5164 lineage).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | panic-path contract suites | fail-closed panic-path posture maintained |
| C-02 | AC-2 | Functional | runtime test decomposition + selector parity contracts | stable module and command-surface behavior |
| C-03 | AC-3 | Integration | docs/governance contract suites | deterministic drift detection in CI-compatible lanes |

## Test Mapping
- Panic-path: `check_no_production_expect` contract lanes and docs parity tests (from #3934 chain).
- Test-surface: `main_module_extraction_contract`, `main_tests_command_surface_parity_contract`, `main_tests_surface_budget_contract`, `node_test_surface_ownership_docs`.

## Success Metrics
- Epic acceptance criteria satisfied with merged child story evidence.
