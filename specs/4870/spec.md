# Spec — Issue #4870

- Title: Task: ratchet CI shell-surface budget and shell-to-Rust trajectory thresholds
- Parent: - Parent story: #4863
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Reviewed
- Priority: P1

## Objective

Implement downward-only threshold ratchets and deterministic reason-code policy outputs for shell-surface governance.

## Problem Statement

This task executes a bounded plan slice needed to reduce shell surface while preserving deterministic contracts.

## Scope

In scope:
- Issue-defined implementation slice and deterministic behavior contracts.
- Conformance and regression checks mapped to acceptance criteria.
- Shell-surface governance markers where script/workflow surface changes.

Out of scope:
- Unrelated runtime/product features.
- Non-deterministic policy behavior.

## Acceptance Criteria

- AC-1: CI emits deterministic reason markers for all threshold failures.
- AC-2: Threshold ratchets enforce non-regression toward shell<Rust sustainment.
- AC-3: Fast-gate remains within bounded runtime budget after checks are added.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
