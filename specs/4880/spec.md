# Spec — Issue #4880

- Title: Subtask: implement declarative_policy_checker.py and schema conformance contracts
- Parent: - Parent task: #4868
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Reviewed
- Priority: P1

## Objective

Build declarative checker core, schema, and validation tooling.

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

- AC-1: Declarative checker validates schema and returns deterministic reason outputs.
- AC-2: Conformance tests cover success/failure/taxonomy mismatch paths.
- AC-3: Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
