# Spec — #4223 Task: Admission/Backpressure Evidence Convergence Checker

Status: Implemented
Priority: P1
Parent: #4220
Milestone: R27.24 Async API concurrency and admission-backpressure governance

## Problem Statement

Service API axum overload policy emits deterministic reason mapping, but there is no dedicated evidence-convergence checker validating lineage and promotion reason mapping consistency across lane and policy artifacts.

## Scope

In scope:
- Add admission/backpressure evidence-convergence checker for service-api-axum artifacts.
- Validate required links and payload consistency across contract lane report, policy report, and source report.
- Fail closed with deterministic reason taxonomy for missing links, payload tamper, and reason-mapping drift.
- Integrate convergence checker into axum contract lane and CI/docs contract surfaces.

Out of scope:
- Service API runtime behavior redesign.
- External orchestration changes.

## Acceptance Criteria

AC-1: Evidence checker validates required links across lane report, policy report, and source report.

AC-2: Missing or tampered artifacts fail closed with deterministic reason codes.

AC-3: Promotion decision reason mapping remains deterministic and is verified by convergence checks.

AC-4: Contract-lane output and docs/tests include convergence checker commands and marker taxonomy.

## Conformance Cases

- C-01 (AC-1, Functional): baseline axum lane + policy artifacts converge to `GO` with `service_api_axum_evidence_convergence_status=verified`.
- C-02 (AC-2, Regression): missing source-report link fails with `service_api_axum_evidence_link_missing:source_report_file`.
- C-03 (AC-2, Regression): payload tamper fails with `service_api_axum_evidence_payload_tamper_detected:<field>`.
- C-04 (AC-3, Integration): tampered promotion reason mapping fails with `service_api_axum_promotion_decision_reason_mapping_mismatch`.
- C-05 (AC-4, Docs): CI/docs contract tests enforce convergence commands and taxonomy markers.
