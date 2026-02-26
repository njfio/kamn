# Spec: Issue #6013 - Refresh production expect() baseline fixture

- Issue: #6013
- Status: Implemented
- Type: task
- Priority: P2
- Area: qa
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26

## Problem Statement
The production `expect()` surface policy test fails on latest `main` because baseline fixture values are stale after merged changes increased tracked production file and `expect()` counts.

## Scope
In scope:
- Refresh `fixtures/ci/production_expect_surface_baseline.env` with current census values.
- Keep threshold policy strict and unchanged.
- Verify the policy test is green.

Out of scope:
- Policy threshold relaxation.
- Broad `expect()` conversion/removal.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: Baseline fixture matches current tracked production source census.
- AC-2: `production_expect_surface_policy` test passes on current main-derived branch.
- AC-3: Threshold fixture remains strict (`allowed_expect_delta_max=0`).

## Conformance Cases
- C-01 (Unit, AC-1): baseline fixture values equal measured values from current census.
- C-02 (Functional, AC-2): `functional_production_expect_surface_non_regression_gate` passes.
- C-03 (Regression, AC-3): threshold fixture stays unchanged and strict.

## Success Metrics / Observable Signals
- CI contract for production `expect()` non-regression is green on updated baseline.
