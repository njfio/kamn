# Spec — #4402 Task: Deterministic Fuzz-Concurrency Checker + Deep-Lane Boundary Governance

Status: Reviewed
Priority: P1
Parent: #4399
Milestone: R27.36 Deep validation hardening, concurrency safety, and observability-emission governance

## Problem Statement

Fuzz/concurrency validation promotion checks need deterministic failure classification for seed/race drift and explicit CI smoke versus local-heavy deep-lane boundary governance.

## Scope

In scope:
- Deterministic reason mapping for fuzz seed replay and concurrency race replay classification.
- Explicit CI smoke/local-heavy boundary markers in invariant-fuzz-concurrency summary/policy reports.
- Fail-closed policy checks for boundary marker drift.
- Tests + docs parity updates for boundary and taxonomy outputs.

Out of scope:
- Always-on heavy fuzz or stress drills in CI fast-gate.
- New runtime deep-lane orchestration wrappers.

## Acceptance Criteria

AC-1: Fuzz seed regression and concurrency race misclassification drift fails closed with deterministic reason codes.

AC-2: Invariant-fuzz-concurrency policy outputs include deterministic CI smoke/local-heavy boundary markers and normalized reason values.

AC-3: Policy checker rejects boundary marker drift with deterministic fail-closed reason codes.

AC-4: CI strategy docs include explicit invariant-fuzz-concurrency boundary marker and fail-closed reason taxonomy updates.

## Conformance Cases

- C-01 (AC-1, Functional): seed replay count tamper yields deterministic `*_replay_test_count_invalid` reason.
- C-02 (AC-1, Regression): concurrency lane fail + mismatched reason payload yields deterministic contract mismatch reasons.
- C-03 (AC-2, Integration): pass path emits boundary status/profile/mode markers and `reason_codes_value=none`.
- C-04 (AC-3, Regression): boundary marker tamper yields deterministic boundary mismatch reason.
- C-05 (AC-4, Docs): `docs/ci/strategy.md` reflects boundary markers + fail-closed reasons.
