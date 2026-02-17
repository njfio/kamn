# Spec — #4238 Task: Crash-Replay Evidence Convergence Checker

Status: Reviewed
Priority: P1
Parent: #4235
Milestone: R27.25 Persistent journal replay and checkpoint-integrity governance

## Problem Statement

Promotion decisions currently validate crash-recovery lane and policy artifacts, but there is no dedicated evidence-convergence checker that verifies lineage links and deterministic reason mapping across journal/checkpoint/recovery artifacts.

## Scope

In scope:
- Add a crash-replay evidence-convergence checker for sqlite crash-recovery contract artifacts.
- Ensure missing/tampered evidence fails closed with deterministic reason taxonomy markers.
- Ensure promotion decision reason mapping is deterministic and convergent.
- Update docs and tests to pin checker and marker contracts.

Out of scope:
- Storage engine changes.
- New runtime crash-recovery execution pathways.

## Acceptance Criteria

AC-1: Convergence checker validates required report/policy/source evidence links.

AC-2: Missing or tampered evidence artifacts fail closed with deterministic reason codes.

AC-3: Promotion decision reason mapping remains deterministic across runs and is verified by convergence checks.

AC-4: Docs and contract tests include crash-replay evidence convergence commands and marker taxonomy.

## Conformance Cases

- C-01 (AC-1, Functional): baseline contract-lane + policy artifacts converge to `GO` with `evidence_convergence_status=verified`.
- C-02 (AC-2, Regression): missing source-report linkage fails with `sqlite_crash_replay_evidence_link_missing:source_report_file`.
- C-03 (AC-2, Regression): tampered policy/report fields fail with `sqlite_crash_replay_evidence_payload_tamper_detected:<field>`.
- C-04 (AC-3, Integration): tampered promotion reason mapping fails with `sqlite_crash_replay_promotion_decision_reason_mapping_mismatch`.
- C-05 (AC-4, Docs): CI/docs contract tests enforce convergence checker commands and taxonomy markers.
