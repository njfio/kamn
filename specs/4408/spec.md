# Spec — #4408 Subtask: Fuzz-Concurrency Reason Taxonomy + CI/Local Boundary Enforcement

Status: Reviewed
Priority: P1
Parent: #4402
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Invariant-fuzz-concurrency policy enforcement lacks explicit fail-closed CI smoke/local-heavy boundary marker validation and deterministic reason taxonomy coverage for boundary drift.

## Scope

In scope:
- Add deterministic boundary marker fields to invariant-fuzz-concurrency summary report payload.
- Enforce boundary marker parity in policy checker with deterministic fail-closed reasons.
- Keep normalized reason value and expected/observed mapping deterministic across pass/fail.
- Update CI strategy docs with boundary marker + reason taxonomy contracts.

Out of scope:
- Adding new heavy deep-lane orchestration scripts.

## Acceptance Criteria

AC-1: Policy checker validates CI/local boundary marker parity with deterministic fail-closed reasons.

AC-2: Summary and policy outputs emit stable boundary markers on pass path.

AC-3: Boundary tamper cases fail with deterministic reason markers and `NO-GO`.

AC-4: CI strategy docs include invariant-fuzz-concurrency boundary marker/reason taxonomy updates.

## Conformance Cases

- C-01 (AC-1, Functional): tampered boundary status marker fails closed deterministically.
- C-02 (AC-2, Integration): pass path emits boundary status/profile/mode markers.
- C-03 (AC-3, Regression): boundary drift contributes deterministic policy reason code ordering.
- C-04 (AC-4, Docs): docs reflect new marker contracts.
