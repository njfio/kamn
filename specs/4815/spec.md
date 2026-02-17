# Spec — Issue #4815

- Title: Task: introduce declarative policy-checker framework and migrate eligible contracts
- Parent: Parent story: #4809
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Implement phase 6 declarative policy-checker architecture and migrate first eligible contract cohort.

## Problem Statement

Current script surface includes large duplicated boilerplate and uneven governance boundaries that increase maintenance burden.

## Scope

In scope:
- phase-aligned implementation and regression checks
- deterministic reason-taxonomy and compatibility markers where applicable
- bounded CI/runtime governance requirements

Out of scope:
- unrelated runtime feature delivery
- non-deterministic policy behavior

## Acceptance Criteria

- AC-1: Declarative checker emits deterministic output equivalent to migrated imperative checks.
- AC-2: First migration cohort demonstrates measurable Python/script surface reduction.
- AC-3: Regression suites catch taxonomy/reason-mapping drift.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence and fail-closed reasons.
- C-02: verify AC-2 with deterministic pass/fail evidence and fail-closed reasons.
- C-03: verify AC-3 with deterministic pass/fail evidence and fail-closed reasons.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Shell-surface reduction or containment impact is explicitly measurable for this scope.
