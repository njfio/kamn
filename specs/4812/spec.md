# Spec — Issue #4812

- Title: Task: replace hardcoded dispatcher mapping and eliminate tiny exec wrappers
- Parent: Parent story: #4807
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Implement phases 1-2 by switching to data-driven manifest resolution and registry-based wrapper dispatch.

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

- AC-1: Dispatcher resolves manifest/phase without hardcoded wrapper case tables.
- AC-2: <=8-line exec wrapper files are reduced via registry/symlink migration.
- AC-3: Regression checks prove wrapper and dispatch compatibility.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence and fail-closed reasons.
- C-02: verify AC-2 with deterministic pass/fail evidence and fail-closed reasons.
- C-03: verify AC-3 with deterministic pass/fail evidence and fail-closed reasons.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Shell-surface reduction or containment impact is explicitly measurable for this scope.
