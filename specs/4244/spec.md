# Spec — #4244 Subtask: Red Tests for Crash-Replay Evidence Convergence

Status: Reviewed
Priority: P1
Parent: #4238
Milestone: R27.25 Persistent journal replay and checkpoint-integrity governance

## Problem Statement

Crash-replay convergence currently lacks dedicated failing tests for missing linkage and tampered payload acceptance.

## Scope

In scope:
- Add red/negative tests for missing evidence link fields.
- Add red/negative tests for tampered policy/report payload fields.
- Add red/negative tests for promotion decision reason mapping drift.

Out of scope:
- Runtime behavior changes outside convergence validation.

## Acceptance Criteria

AC-1: Missing source-report linkage fails deterministically.

AC-2: Tampered evidence payload fields fail deterministically.

AC-3: Promotion decision reason mapping drift fails deterministically.

## Conformance Cases

- C-01 (AC-1, Regression): missing `source_report_file` yields `sqlite_crash_replay_evidence_link_missing:source_report_file`.
- C-02 (AC-2, Regression): altered schema/decision fields yield `sqlite_crash_replay_evidence_payload_tamper_detected:<field>`.
- C-03 (AC-3, Regression): altered promotion reason code yields `sqlite_crash_replay_promotion_decision_reason_mapping_mismatch`.
