# Spec — #4240 Subtask: Red Tests for Journal Append-Checkpoint Mismatch Rejection

Status: Implemented
Priority: P1
Parent: #4236
Milestone: R27.25 Persistent Journal Replay and Checkpoint-Integrity Governance

## Problem Statement

Current crash-recovery regression fixtures do not explicitly cover WAL append marker drift or append-checkpoint parity mismatch rejection behavior.

## Scope

In scope:
- Add tamper fixtures for WAL append mismatch and append/checkpoint parity mismatch.
- Assert deterministic fail-closed reason markers.

Out of scope:
- Checker implementation redesign.

## Acceptance Criteria

AC-1: WAL append marker drift fails checker with deterministic reason mapping.

AC-2: Append/checkpoint parity mismatch fails checker with deterministic reason mapping.

AC-3: Regression test script remains deterministic and green on baseline.

## Conformance Cases

- C-01 (AC-1): tampered `wal_append_status` fixture fails with `sqlite_crash_recovery_policy_wal_append_status_mismatch`.
- C-02 (AC-2): append/checkpoint parity mismatch fixture fails with `sqlite_crash_recovery_policy_append_checkpoint_parity_mismatch`.
- C-03 (AC-3): baseline run still reports policy `status=ok`, `final_decision=GO`.
