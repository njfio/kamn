# Spec — #4397 Subtask: persistence gate reason taxonomy with CI smoke/local-heavy boundary enforcement

Status: Implemented
Priority: P1
Parent: #4389
Milestone: R27.35 Async API framework hardening, real peer transport, and durable state-store validation governance

## Problem Statement

Persistence promotion gates require deterministic reason taxonomy and explicit CI/local execution-boundary markers.

## Scope

In scope:
- Implement deterministic persistence gate taxonomy outputs.
- Implement CI smoke/local-heavy boundary markers in validation outputs.
- Add fail-closed mismatch reasons for persistence marker drift/tampering.
- Update docs and docs-contract tests with required markers.

Out of scope:
- New deep-lane runtime orchestration.

## Acceptance Criteria

AC-1: Persistence gate reason taxonomy version and reason-code CSV are emitted deterministically.

AC-2: CI smoke/local-heavy boundary markers are emitted and enforced as contracts.

AC-3: Tampered/missing persistence marker values produce deterministic fail-closed mismatch reasons.

AC-4: Documentation and docs-contract tests include new persistence gate markers.

## Conformance Cases

- C-01 (AC-1, Functional): stdout/JSON include persistence taxonomy version and reason-code CSV.
- C-02 (AC-2, Functional): stdout/JSON include CI/local boundary marker fields.
- C-03 (AC-3, Regression): tampered taxonomy/marker values are rejected with deterministic reasons.
- C-04 (AC-4, Integration): release checklist + CI strategy docs include required persistence markers and tests pass.
