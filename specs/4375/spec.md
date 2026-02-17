# Spec — #4375 Subtask: RED tests for production-mode in-memory provider acceptance

Status: Reviewed
Priority: P1
Parent: #4371
Milestone: R27.34 Live Kolme provider integration, native secp256k1 signing, and end-to-end validation governance

## Problem Statement
Regression tests must fail immediately when production-mode checks accept in-memory provider paths.

## Scope
In scope:
- RED tamper tests for in-memory provider marker acceptance.
- Deterministic failure reason assertions.

Out of scope:
- New provider runtime implementation.

## Acceptance Criteria
AC-1: Tests fail when in-memory provider references are accepted in production-mode path.

AC-2: Failure output includes deterministic reason mapping.

AC-3: RED->GREEN flow is demonstrated in PR evidence.

## Conformance Cases
- C-01 (AC-1, Functional): injected in-memory provider marker causes policy NO-GO.
- C-02 (AC-2, Regression): failure includes `runtime_commit_in_memory_provider_reference_detected`.
- C-03 (AC-3, Integration): lane test passes after implementation.
