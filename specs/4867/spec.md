# Spec — Issue #4867

- Title: Task: deploy shared shell test harness and JSON helper migration waves (phases 4-5)
- Parent: - Parent story: #4861
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Reviewed
- Priority: P1

## Objective

Adopt reusable shell test harness and JSON emit/write helpers across high-duplication scripts.

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

- AC-1: Shared harness helpers replace duplicated assertion/usage/test setup logic.
- AC-2: JSON helper utilities replace manual ad-hoc JSON string construction in target scripts.
- AC-3: Docs-contract tests guard required helper markers.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
