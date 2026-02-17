# Spec — #4406 Subtask: Deterministic Invariant Failure Reason Mapping

Status: Reviewed
Priority: P1
Parent: #4401
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Invariant policy enforcement lacks normalized deterministic reason-taxonomy output and strict expected/observed reason mapping checks required by release-governance workflows.

## Scope

In scope:
- Deterministic reason-taxonomy/version/csv marker constants for invariant policy checks.
- Deterministic mapping from lane/runtime failure conditions to expected reason codes.
- Normalized pass/fail policy evidence markers emitted by checker.
- Stable taxonomy/evidence fields in combined invariant lane report payload.

Out of scope:
- New deep-lane orchestration or formal verification tooling.

## Acceptance Criteria

AC-1: Invariant failure reason mapping is deterministic for lane/runtime failure classes.

AC-2: Policy checker emits stable taxonomy and reason-evidence markers on pass and fail paths.

AC-3: Integration tests validate deterministic mapping and fail-closed enforcement for tampered reports.

## Conformance Cases

- C-01 (AC-1, Functional): expected reason mapping for lane/runtime-derived failures is deterministic.
- C-02 (AC-2, Integration): pass path emits taxonomy/version/csv/value markers and `GO` decision.
- C-03 (AC-2, Integration): fail path emits taxonomy/version/csv/value markers and `NO-GO` decision.
- C-04 (AC-3, Regression): tampered lane acceptance and taxonomy drift reports are rejected with deterministic reason codes.
