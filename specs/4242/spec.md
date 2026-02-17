# Spec — #4242 Subtask: Red Tests for Replay Taxonomy Drift and Runbook Marker Divergence

Status: Reviewed
Priority: P1
Parent: #4237
Milestone: R27.25 Persistent journal replay and checkpoint-integrity governance

## Problem Statement

Replay-idempotency taxonomy/runbook parity is not currently enforced by deterministic red tests in
the sqlite crash-recovery policy checker path.

## Scope

In scope:
- Add failing tests first for replay taxonomy mapping drift.
- Add failing tests first for replay runbook marker divergence.
- Assert deterministic fail-closed reason markers.

Out of scope:
- Policy implementation changes beyond what is required to keep tests red before implementation.

## Acceptance Criteria

AC-1: Red test fails when replay taxonomy mapping marker is tampered.
AC-2: Red test fails when replay runbook taxonomy marker set diverges.
AC-3: Red test fails when runbook marker parity markers are missing.

## Conformance Cases

- C-01 (Regression): report tamper on replay taxonomy mapping status yields
  `replay_idempotency_taxonomy_mapping_drift_detected`.
- C-02 (Regression): report tamper on replay runbook taxonomy version yields
  `sqlite_crash_recovery_policy_replay_idempotency_runbook_reason_taxonomy_version_mismatch`.
- C-03 (Regression): runbook marker removal yields `runbook_marker_parity_mismatch`.

## Success Signals

- Tests demonstrate failures before implementation, then pass unchanged after implementation.
