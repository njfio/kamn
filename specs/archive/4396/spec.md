# Spec — #4396 Subtask: RED tests for persistence tamper/freshness drift and incomplete evidence acceptance

Status: Implemented
Priority: P1
Parent: #4389
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

Persistence evidence regressions must fail deterministically before implementation changes land.

## Scope

In scope:
- Add failing test assertions for required persistence marker/taxonomy outputs.
- Add failing tamper-path assertions for marker drift rejection.

Out of scope:
- Production behavior changes not needed for RED phase.

## Acceptance Criteria

AC-1: Tests fail when required persistence taxonomy/marker fields are absent.

AC-2: Tests fail when tampered/incomplete evidence is accepted.

AC-3: Regression tests cover freshness/tamper drift in a deterministic way.

## Conformance Cases

- C-01 (AC-1, Functional): missing persistence taxonomy marker fails test suite.
- C-02 (AC-1, Functional): missing persistence boundary marker fails test suite.
- C-03 (AC-2, Regression): tampered marker in report path is rejected with deterministic reason.
- C-04 (AC-2, Regression): incomplete evidence marker set is rejected.
- C-05 (AC-3, Regression): repeated runs preserve deterministic failure message matching.
