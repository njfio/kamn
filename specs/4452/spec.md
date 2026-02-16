# Spec: Issue #4452

Status: Implemented
Issue: #4452
Parent: #4447
Milestone: R27.39 Runtime decomposition, panic-free execution, and dependency-license governance
Priority: P1

## Problem Statement

`main_tests` decomposition and test-surface structural budget governance need explicit
fail-closed contracts so drift is detected deterministically before merge. Current
coverage does not pin a dedicated decomposition/budget drift contract surface for this
task lineage.

## Scope

In scope:
- RED/green contract tests for `main_tests` decomposition drift.
- RED/green contract tests for structural budget governance markers.
- Documentation anchors for decomposition drift and budget-failure cases.

Out of scope:
- Large-scale historical test rewrite.
- Runtime behavior changes unrelated to decomposition/budget contracts.
- CI workflow topology changes.

## Acceptance Criteria

AC-1:
Given `main_tests` decomposition boundaries, when decomposition contract tests run, then
tests fail closed if domain-module boundaries drift or inline monolith regressions return.

AC-2:
Given structural budget governance references, when docs contract tests run, then tests
fail closed when required structural budget markers/commands are missing.

AC-3:
Given targeted integration checks, when decomposition and docs tests run together, then
deterministic structure-governance coverage remains stable.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `main_module_extraction_contract_main_tests_decomposition_and_budget_markers_remain_stable`
  - Expectation: `main_tests.rs` remains module-scoped and retains decomposition
    delegation markers.

- C-02 (AC-2, Conformance/Regression):
  - Test: `doc_contains_main_tests_decomposition_drift_and_budget_markers`
  - Expectation: docs include deterministic taxonomy/budget markers for structure governance.

- C-03 (AC-3, Integration/Regression):
  - Tests:
    - `cargo test -p kamn-node --test main_module_extraction_contract`
    - `cargo test -p kamn-core --test testing_structure_docs`
  - Expectation: decomposition and docs contracts pass together and fail closed on drift.

## Success Metrics / Observable Signals

- New decomposition/budget contract tests are deterministic and pass repeatedly.
- Drift in main-tests module boundaries or required structure docs markers triggers
  predictable failures.
- Structural budget governance commands are pinned in docs for low-cost CI replay.
