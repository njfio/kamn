# Spec: Issue #6080 - Mark #6075/#6076 specs Implemented

- Issue: #6080
- Status: Reviewed
- Type: task
- Priority: P2
- Area: governance
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6075

## Problem Statement
Issues #6075 and #6076 are closed and merged, but their spec headers still report `Status: Reviewed` rather than `Status: Implemented`, creating lifecycle drift.

## Scope
In scope:
- Update `specs/6075/spec.md` and `specs/6076/spec.md` status markers to `Implemented`.
- Record closure evidence comments on #6075/#6076 with implemented status markers.

Out of scope:
- Runtime code behavior changes.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: `specs/6075/spec.md` header status is `Implemented`.
- AC-2: `specs/6076/spec.md` header status is `Implemented`.
- AC-3: #6075/#6076 closure comments explicitly reference implemented spec markers.

## Conformance Cases
- C-01 (Conformance, AC-1): `specs/6075/spec.md` shows `- Status: Implemented`.
- C-02 (Conformance, AC-2): `specs/6076/spec.md` shows `- Status: Implemented`.
- C-03 (Regression, AC-3): issue comments include Outcome/PR/Milestone/Spec/Test/Conformance/Mutants closure markers.

## Success Metrics / Observable Signals
- Closed issues and in-repo spec statuses are lifecycle-consistent.
- Milestone artifact index includes #6080 lifecycle artifacts.
