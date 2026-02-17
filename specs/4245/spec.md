# Spec — #4245 Subtask: Replay Evidence Convergence Verifier + Reason Mapping

Status: Reviewed
Priority: P1
Parent: #4238
Milestone: R27.25 Persistent journal replay and checkpoint-integrity governance

## Problem Statement

Promotion approvals need deterministic mapping between crash-replay policy reasons and evidence-convergence outcomes.

## Scope

In scope:
- Implement convergence verifier command surface for sqlite crash-recovery artifacts.
- Enforce deterministic reason taxonomy and normalized output markers.
- Validate promotion reason mapping against policy reason sets.

Out of scope:
- Broader deployment orchestration changes.

## Acceptance Criteria

AC-1: Convergence verifier accepts valid report/policy/source-report lineage with `GO`.

AC-2: Convergence verifier rejects missing/tampered evidence deterministically.

AC-3: Promotion decision reason mapping is deterministic and validated by convergence verifier.

## Conformance Cases

- C-01 (AC-1, Functional): baseline artifacts produce `status=ok`, `final_decision=GO`, and `reason_codes_value=none`.
- C-02 (AC-2, Regression): missing linkage and payload tamper produce deterministic `NO-GO` reason markers.
- C-03 (AC-3, Integration): policy reason-code drift produces deterministic mapping mismatch marker.
