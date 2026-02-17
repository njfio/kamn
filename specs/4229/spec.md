# Spec — #4229 Subtask: Red Tests for Admission/Backpressure Evidence Convergence

Status: Implemented
Priority: P1
Parent: #4223
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement

Admission/backpressure evidence convergence requires deterministic test coverage for missing links and tampered payload rejection.

## Scope

In scope:
- Add red/green fixture tests for evidence-link completeness and tamper rejection.
- Verify deterministic fail-closed reason output across repeated runs.

Out of scope:
- Runtime lane behavior changes beyond convergence-check surface.

## Acceptance Criteria

AC-1: Missing required evidence links fail convergence tests deterministically.

AC-2: Tampered convergence payloads fail with deterministic reason markers.

AC-3: Regression tests preserve deterministic reason ordering/stability.

## Conformance Cases

- C-01 (AC-1, Regression): missing source link fails with `service_api_axum_evidence_link_missing:source_report_file`.
- C-02 (AC-2, Regression): tampered payload field fails with `service_api_axum_evidence_payload_tamper_detected:<field>`.
- C-03 (AC-2/AC-3, Regression): tampered reason mapping fails with `service_api_axum_promotion_decision_reason_mapping_mismatch`.
