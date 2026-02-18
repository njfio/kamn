# Spec — Issue #4861

- Title: Story: consolidate wave/test/json shell boilerplate with reusable harnesses (phases 3-5)
- Parent: - Program epic: #3812
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Implemented
- Priority: P1

## Objective

As a maintainer, I need reusable wave/test/json harnesses so that shell contract lanes remain readable and maintainable as coverage grows.

## Problem Statement

Phase 3-5 work still suffers from repeated matrix scripts and manual JSON construction patterns.

## Scope

In scope:
- Issue-defined implementation slice and deterministic behavior contracts.
- Conformance and regression checks mapped to acceptance criteria.
- Shell-surface governance markers where script/workflow surface changes.

Out of scope:
- Unrelated runtime/product features.
- Non-deterministic policy behavior.

## Acceptance Criteria

- AC-1: Wave and wrapper-matrix script families are consolidated into definition-driven runners.
- AC-2: Shared test harness and JSON helpers replace duplicated logic in targeted script sets.
- AC-3: Script LOC and duplicate function counts decline with deterministic evidence markers.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
