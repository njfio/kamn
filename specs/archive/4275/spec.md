# Spec — #4275 Subtask: Websocket Evidence Verifier and Deterministic Promotion Reason Mapping

Status: Implemented
Priority: P1
Parent: #4268
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Policy-level websocket outcomes must project stable promotion decision reasons and be cross-validated against lane evidence artifacts. Missing deterministic mapping risks ambiguous release gating.

## Acceptance Criteria

AC-1: Policy checker emits deterministic promotion decision reason mapping markers.

AC-2: Evidence convergence verifier cross-validates lane report/policy/source summary linkage.

AC-3: Promotion reason mapping mismatch is rejected with deterministic fail-closed reason.

## Conformance Cases

- C-01 (AC-1): successful policy check emits `promotion_decision_reason_code=none`.
- C-02 (AC-1): failing policy check maps to deterministic reason class (e.g. `ci_fast_gate_failed`).
- C-03 (AC-2): valid lane+policy linkage passes convergence with `evidence_convergence_status=verified`.
- C-04 (AC-3): tampered mapping reason code is rejected with `service_api_websocket_promotion_decision_reason_mapping_mismatch`.
