# Spec — Issue #4811

- Title: Task: introduce scripts/lib/common.sh and migrate duplicated shell boilerplate
- Parent: Parent story: #4807
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Implement phase 0 common shell library and migrate duplicated ROOT_DIR/assert/usage/extract patterns in bounded waves.

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

- AC-1: common.sh provides stable shared primitives used by migrated scripts.
- AC-2: Migrated scripts preserve behavior and contract output compatibility.
- AC-3: Boilerplate duplication metrics decline in the migration wave.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence and fail-closed reasons.
- C-02: verify AC-2 with deterministic pass/fail evidence and fail-closed reasons.
- C-03: verify AC-3 with deterministic pass/fail evidence and fail-closed reasons.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Shell-surface reduction or containment impact is explicitly measurable for this scope.
