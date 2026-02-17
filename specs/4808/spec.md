# Spec — Issue #4808

- Title: Story: collapse test/matrix/json shell boilerplate into reusable harnesses
- Parent: Parent epic: #4806
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Execute phases 3-5 by parameterizing wave scripts, introducing shared test harness utilities, and eliminating manual JSON output duplication.

## Problem Statement

Test and output boilerplate dominates shell LOC and increases drift risk between near-identical scripts.

## Scope

In scope:
- wave/matrix script parameterization
- test harness introduction and migration
- common JSON helper rollout

Out of scope:
- changing evidence schemas themselves
- removing contract-lane validation semantics

## Acceptance Criteria

- AC-1: Wave/matrix duplicates are replaced by parameterized runners with equivalent coverage.
- AC-2: Harness migration cuts repeated test setup/assert code while preserving deterministic failures.
- AC-3: Manual JSON emission footprint is materially reduced via shared helper calls.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence and fail-closed reasons.
- C-02: verify AC-2 with deterministic pass/fail evidence and fail-closed reasons.
- C-03: verify AC-3 with deterministic pass/fail evidence and fail-closed reasons.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Shell-surface reduction or containment impact is explicitly measurable for this scope.
