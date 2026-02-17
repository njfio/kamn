# Spec — #4237 Task: Replay Idempotency Taxonomy and Runbook Marker Parity Under Crash Recovery

Status: Reviewed
Priority: P1
Parent: #4234
Milestone: R27.25 Persistent journal replay and checkpoint-integrity governance

## Problem Statement

Sqlite crash-recovery policy validation currently enforces journal replay markers, but it does not
project an explicit replay idempotency taxonomy-to-runbook parity contract. This leaves a gap where
taxonomy drift and runbook marker divergence can pass without deterministic, policy-level reasons.

## Scope

In scope:
- Add replay idempotency taxonomy mapping markers to sqlite crash-recovery run-lane and policy output.
- Enforce runbook marker parity checks in sqlite crash-recovery policy mode.
- Add deterministic fail-closed reasons for replay taxonomy drift and runbook marker divergence.
- Extend contract-lane tests and docs-contract tests for the new marker family.

Out of scope:
- Crash-recovery runtime orchestration redesign.
- Non-sqlite storage policy changes.

## Acceptance Criteria

AC-1: Sqlite crash-recovery run-lane and policy output include deterministic replay idempotency
taxonomy mapping + runbook parity markers.

AC-2: Policy checker fails closed on replay taxonomy drift with deterministic reason codes.

AC-3: Policy checker fails closed when runbook markers diverge from required replay taxonomy markers.

AC-4: Deploy/release docs and docs-contract tests remain synchronized with the new marker/reason set.

## Conformance Cases

- C-01 (AC-1, Functional): policy output contains
  `replay_idempotency_taxonomy_mapping_status=verified`,
  `runbook_marker_parity_status=verified`,
  `replay_idempotency_runbook_reason_taxonomy_version=...`,
  `replay_idempotency_runbook_reason_codes_csv=...`.
- C-02 (AC-2, Regression): tampered replay taxonomy marker in report fails with
  `replay_idempotency_taxonomy_mapping_drift_detected`.
- C-03 (AC-2, Regression): tampered replay runbook taxonomy version fails with
  `sqlite_crash_recovery_policy_replay_idempotency_runbook_reason_taxonomy_version_mismatch`.
- C-04 (AC-3, Regression): runbook missing required replay markers fails with
  `runbook_marker_parity_mismatch`.
- C-05 (AC-4, Integration): docs-contract tests assert deploy/release docs include replay
  taxonomy/runbook parity markers and reasons.

## Success Signals

- Replay idempotency taxonomy drift and runbook marker divergence reject deterministically.
- Contract lane and policy tests capture red/green for the new rejection reasons.
- Runbook and release checklist markers are enforced by docs-contract tests.
