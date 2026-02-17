# Spec — Issue #4806

- Title: Epic: reduce shell script surface and institutionalize shell-to-Rust LOC governance
- Parent: Program epic: #3812
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Deliver a phased shell-surface reduction program that drives shell LOC below Rust LOC while preserving contract-lane determinism and CI safety.

## Problem Statement

Shell surface growth has outpaced Rust significantly; duplication and wrapper proliferation are increasing maintenance cost and policy drift risk.

## Scope

In scope:
- phase-based reduction execution for shared shell libs, dispatcher/wrapper consolidation, test boilerplate reduction, and manifest/policy normalization
- explicit CI and process governance that enforces script-surface budgets and shell:Rust ratio trend targets
- milestone-level backlog and specs enabling traceable, testable execution

Out of scope:
- rewriting domain business logic for non-duplication reasons
- introducing always-on heavy lanes in fast-gate
- non-Kolme architecture redesign unrelated to script-surface containment

## Acceptance Criteria

- AC-1: All reduction phases have scoped child stories/tasks/subtasks with deterministic acceptance criteria.
- AC-2: Milestone governance includes an explicit trajectory from shell:Rust ratio 2.19 toward <1.0 with bounded checkpoints.
- AC-3: Future work contracts include fail-closed checks preventing script-surface regression.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence and fail-closed reasons.
- C-02: verify AC-2 with deterministic pass/fail evidence and fail-closed reasons.
- C-03: verify AC-3 with deterministic pass/fail evidence and fail-closed reasons.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Shell-surface reduction or containment impact is explicitly measurable for this scope.
