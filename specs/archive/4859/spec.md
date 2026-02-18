# Spec — Issue #4859

- Title: Epic: execute shell LOC reduction phases 0-7 and enforce shell-to-Rust sustainment
- Parent: - Program epic: #3812
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Implemented
- Priority: P1

## Objective

Execute docs/plans/2026-02-17-shell-loc-reduction-plan.md phases 0-7 with enforceable governance that keeps shell LOC maintainable and permanently below Rust LOC.

## Problem Statement

Shell surface remains operationally expensive and regression-prone unless reduction phases and sustainment controls are encoded in active backlog contracts with measurable checkpoints.

## Scope

In scope:
- Issue-defined implementation slice and deterministic behavior contracts.
- Conformance and regression checks mapped to acceptance criteria.
- Shell-surface governance markers where script/workflow surface changes.

Out of scope:
- Unrelated runtime/product features.
- Non-deterministic policy behavior.

## Acceptance Criteria

- AC-1: Child stories/tasks/subtasks map to all plan phases (0-7) plus sustainment governance.
- AC-2: All child issues include milestone links and per-issue spec artifacts (`spec.md`, `plan.md`, `tasks.md`).
- AC-3: Governance controls require shell-surface delta declarations and fail closed on ratio/script budget regressions.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
