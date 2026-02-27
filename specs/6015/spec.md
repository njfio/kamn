# Spec: Issue #6015 - Correct production expect() census scope

- Issue: #6015
- Status: Implemented
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26

## Problem Statement
`production_expect_surface_policy` currently overcounts production panic surface by including test-only source files and `#[cfg(test)]` blocks from files under `src/`, which inflates audit numbers and obscures true production risk.

## Scope
In scope:
- Exclude test-only source files from census (e.g., `main_tests`, `runtime_tests*`, `tests.rs`, `test_*`, `*_tests`).
- Exclude `#[cfg(test)]` items when counting `.expect(` in otherwise production files.
- Keep deterministic `git ls-tree` source enumeration.
- Refresh baseline fixture to corrected scope values.

Out of scope:
- Converting/removing `expect()` in production logic.
- Relaxing threshold policy.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: Census no longer counts `expect()` from `#[cfg(test)]` blocks.
- AC-2: Census excludes test-only source files and `kamn-e2e-harness`.
- AC-3: Policy test is green with updated baseline under strict `allowed_expect_delta_max=0`.

## Conformance Cases
- C-01 (Unit, AC-1): regression fixture verifies `expect()` inside `#[cfg(test)] mod tests` is excluded while production `expect()` is counted.
- C-02 (Functional, AC-2): `functional_production_expect_surface_non_regression_gate` passes with corrected census scope.
- C-03 (Regression, AC-3): threshold fixture remains strict and unchanged.

## Success Metrics / Observable Signals
- Reported production `expect()` count reflects runtime code only.
- Non-regression gate still fail-closed for real production-surface growth.
