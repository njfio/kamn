# Spec — #4300 Subtask: RED Tests for Retry Envelope Exhaustion and Deterministic Fail-Closed Outputs

Status: Implemented
Priority: P1
Parent: #4296
Milestone: R27.29 Observability, transport resilience, and TLS governance convergence

## Problem Statement

Retry envelope/fail-closed behavior can regress without deterministic red tests that assert bounded
reconnect/backoff policy and stable reason outputs.

## Scope

In scope:
- RED tests for retry-envelope exhaustion fail-closed markers.
- RED tests for reconnect/backoff bound drift rejection.
- RED tests for deterministic reason taxonomy/reason-csv marker drift.

Out of scope:
- Runtime transport protocol redesign.

## Acceptance Criteria

AC-1: Tests fail when retry-envelope exhaustion fail-closed markers are missing or incorrect.

AC-2: Tests fail when reconnect/backoff bound markers drift from deterministic contract.

AC-3: Tests fail when reason taxonomy/version or reason-csv markers drift.

## Conformance Cases

- C-01 (AC-1, Regression): tampered summary report with exhaustion marker drift is rejected.
- C-02 (AC-2, Regression): tampered reconnect-attempt bound marker is rejected.
- C-03 (AC-2, Regression): tampered backoff-window bound marker is rejected.
- C-04 (AC-3, Regression): taxonomy-version drift fails with deterministic reason.
- C-05 (AC-3, Regression): reason-csv drift fails with deterministic reason.

## Success Signals

- RED tests capture all intended fail-closed drift paths deterministically.
