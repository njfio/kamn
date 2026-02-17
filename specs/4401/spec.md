# Spec — #4401 Task: Property-Based Invariant Checks for Runtime and Promotion Gate Contracts

Status: Reviewed
Priority: P1
Parent: #4399
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

The invariant/fuzz/concurrency contract lane currently validates baseline schema and replay counters, but deterministic failure-reason mapping and stable policy evidence outputs are incomplete for promotion-governance workflows.

## Scope

In scope:
- Add red tests for invariant violation acceptance and unstable evidence outputs (#4405).
- Implement deterministic invariant failure reason mapping and normalized policy evidence outputs (#4406).
- Keep CI-smoke scope bounded and compatible with the existing invariant lane contract surface.

Out of scope:
- Formal verification and unbounded local-heavy deep validation.

## Acceptance Criteria

AC-1: Invariant violation acceptance paths are covered by deterministic failing tests.

AC-2: Invariant policy checker emits deterministic reason-taxonomy evidence outputs on both pass and fail paths.

AC-3: Invariant policy checker enforces stable reason mapping between lane status/runtime budget and fail-closed reason codes.

AC-4: Runtime invariant/fuzz/concurrency lane and policy contracts remain green in CI-smoke scope.

## Conformance Cases

- C-01 (AC-1, Regression): tampered report that marks lane failure as pass is rejected by policy checker with deterministic mismatch markers.
- C-02 (AC-1, Regression): tampered report with unstable taxonomy/evidence markers is rejected fail-closed.
- C-03 (AC-2, Functional): checker pass output includes deterministic taxonomy + reason marker set.
- C-04 (AC-3, Integration): checker validates deterministic expected reason mapping for lane/runtime failure conditions.
- C-05 (AC-4, Integration/Performance): invariant contract lane + policy checker tests remain green under bounded CI-smoke runtime.
