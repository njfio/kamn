# Spec — #4411 Subtask: Red Tests for Telemetry Gate Evidence Convergence Gaps

Status: Reviewed
Priority: P1
Parent: #4404
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Telemetry policy tests do not currently force fail-closed behavior for incomplete evidence-link wiring and partial-evidence acceptance in run-mode reports.

## Scope

In scope:
- Add RED policy-checker tests for missing required telemetry evidence links.
- Add RED policy-checker tests for partial artifact wiring accepted as GO.
- Keep tests deterministic and bounded.

Out of scope:
- Implementing policy-checker fixes (handled in #4412).

## Acceptance Criteria

AC-1: Tests fail when run-mode telemetry reports omit required evidence links.

AC-2: Tests fail when run-mode telemetry reports include partial evidence wiring.

AC-3: Regression tests preserve deterministic fail-closed reason markers for telemetry policy failures.

## Conformance Cases

- C-01 (AC-1, Functional): missing evidence-link keys are rejected.
- C-02 (AC-2, Functional): partial evidence-link maps are rejected.
- C-03 (AC-3, Regression): existing tamper fail-closed checks remain green.

