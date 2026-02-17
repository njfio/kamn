# Spec — Issue #4864

- Title: Task: migrate scripts to common.sh and remove duplicated helper boilerplate (phase 0)
- Parent: - Parent story: #4860
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Reviewed
- Priority: P1

## Objective

Create migration waves for shared shell primitives (`common.sh`) and remove duplicated ROOT_DIR/usage/assert/extract helpers.

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

- AC-1: common.sh is sourced by target script families without behavior drift.
- AC-2: Duplicated helper counts drop measurably in migrated sets.
- AC-3: Compatibility tests remain green with deterministic reason markers.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
