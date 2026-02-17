# Spec — Issue #4817

- Title: Task: add fail-closed CI gates for shell-to-Rust ratio and script budget thresholds
- Parent: Parent story: #4810
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Add deterministic CI governance that blocks script-surface regressions and enforces shell/Rust ratio trajectory.

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

- AC-1: CI emits deterministic reason taxonomy for ratio/budget failures.
- AC-2: Fast-gate integration remains bounded and low-cost.
- AC-3: Telemetry artifacts track ratio trend and gate outcomes per run.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence and fail-closed reasons.
- C-02: verify AC-2 with deterministic pass/fail evidence and fail-closed reasons.
- C-03: verify AC-3 with deterministic pass/fail evidence and fail-closed reasons.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Shell-surface reduction or containment impact is explicitly measurable for this scope.
