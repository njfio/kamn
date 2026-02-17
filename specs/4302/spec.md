# Spec — #4302 Subtask: Add RED Tests for API/Runtime/Kolme Correlation Schema Drift and Propagation Gaps

Status: Implemented
Priority: P1
Parent: #4297
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

The unified local-heavy observability checker lacks regression coverage for correlation-schema drift
and cross-surface propagation mismatches. Without RED tests, drift can merge undetected.

## Scope

In scope:
- Add failing tests (before implementation) for:
  - missing/invalid correlation schema markers
  - schema version drift
  - API/runtime/Kolme correlation propagation mismatch
- Keep tests within existing unified local-heavy test scripts.

Out of scope:
- Implementing policy logic itself.
- Introducing new test executables.

## Acceptance Criteria

AC-1: Tests fail when correlation schema version markers drift.

AC-2: Tests fail when required correlation field markers are missing/invalid.

AC-3: Tests fail when API/runtime/Kolme correlation-id propagation markers diverge.

## Conformance Cases

- C-01 (AC-1, Regression): tampered report schema marker returns deterministic schema-drift reason.
- C-02 (AC-2, Regression): tampered required correlation marker returns deterministic required-field reason.
- C-03 (AC-3, Regression): tampered API/runtime/Kolme correlation-id parity returns deterministic propagation-mismatch reason.

## Success Signals

- RED tests fail on current implementation before checker updates.
- Test failures clearly name deterministic reason codes expected by the policy taxonomy.
