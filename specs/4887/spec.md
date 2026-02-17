# Spec — Issue #4887

- Title: Subtask: add docs-contract and PR-template checks for shell governance markers
- Parent: - Parent task: #4871
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Reviewed
- Priority: P1

## Objective

Add deterministic docs-contract tests validating required shell governance markers in docs/templates.

## Problem Statement

This subtask delivers a focused implementation slice required to satisfy its parent task acceptance criteria.

## Scope

In scope:
- Issue-defined implementation slice and deterministic behavior contracts.
- Conformance and regression checks mapped to acceptance criteria.
- Shell-surface governance markers where script/workflow surface changes.

Out of scope:
- Unrelated runtime/product features.
- Non-deterministic policy behavior.

## Acceptance Criteria

- AC-1: Docs-contract tests fail when required shell governance markers are missing.
- AC-2: PR template guidance enforces shell LOC delta and mitigation linkage in review.
- AC-3: Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
