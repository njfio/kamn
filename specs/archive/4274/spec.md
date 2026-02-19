# Spec — #4274 Subtask: RED Tests for Websocket Evidence Convergence Completeness/Tamper Rejection

Status: Implemented
Priority: P1
Parent: #4268
Milestone: R27.27 API protocol compliance and websocket-session governance

## Problem Statement

Without explicit RED regression coverage, missing evidence links and payload tamper paths can regress silently in websocket convergence checks.

## Acceptance Criteria

AC-1: Missing required evidence links fail convergence checks with deterministic reason markers.

AC-2: Tampered evidence payloads fail convergence checks with deterministic reason markers.

AC-3: Repeated failing runs preserve deterministic reason-code ordering.

## Conformance Cases

- C-01 (AC-1): remove `source_report_file`; checker returns `service_api_websocket_evidence_link_missing:source_report_file`.
- C-02 (AC-2): mutate payload shape/type; checker returns `service_api_websocket_evidence_payload_tamper_detected:<field>`.
- C-03 (AC-2): mutate promotion reason mapping; checker returns `service_api_websocket_promotion_decision_reason_mapping_mismatch`.
- C-04 (AC-3): repeated missing-link runs emit identical reason-code ordering.
