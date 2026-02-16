# Spec: Issue #4462

Status: Implemented
Issue: #4462
Parent: #4459
Milestone: R27.40 Compliance audit-trail integrity, SLO-governance, and incident-readiness contracts
Priority: P1

## Problem Statement

Release go/no-go evidence currently lacks an explicit SLO-threshold convergence gate on top of
deployment SLO/rollback policy evidence. Without deterministic SLO gate convergence checks,
threshold drift and gate mismatch acceptance can bypass promotion safety.

## Scope

In scope:
- Add optional SLO policy gate ingestion to go/no-go generator/checker contract.
- Fail closed on missing/invalid/stale/non-pass SLO policy evidence.
- Emit deterministic SLO policy gate taxonomy and normalized reason-code markers.
- Add tests and docs updates for SLO threshold/gate convergence contracts.

Out of scope:
- Global SRE platform/tooling rollout.
- New runtime telemetry backend.

## Acceptance Criteria

AC-1:
Given go/no-go generation with SLO policy evidence enabled, when threshold policy evidence is
missing, invalid, stale, or non-passing, then final decision fails closed deterministically.

AC-2:
Given SLO policy evidence is enabled, when generator/checker executes, then deterministic
`slo_policy_reason_taxonomy_version`, `slo_policy_reason_codes_csv`, and
`slo_policy_reason_codes_value` markers are emitted.

AC-3:
Given tampered SLO policy gate payloads, when checker validates bundle, then checker fails closed
with deterministic convergence mismatch.

AC-4:
Given docs contract tests run, when observability/release docs are validated, then SLO threshold and
policy-gate taxonomy references are present and drift-protected.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: missing/invalid/stale/non-pass SLO policy evidence yields deterministic NO-GO.

- C-02 (AC-2, Unit/Functional):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: deterministic SLO taxonomy/csv/value outputs for GO/NO-GO paths.

- C-03 (AC-3, Regression/Conformance):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: tampered `slo_policy_gate` payload fails with convergence mismatch.

- C-04 (AC-3, Integration):
  - Test: `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - Expectation: contract lane validates SLO policy gate convergence.

- C-05 (AC-4, Docs):
  - Test: `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - Expectation: release checklist includes SLO gate commands/markers and regression policy.

- C-06 (AC-4, Docs):
  - Test: `cargo test -p kamn-core --test observability_schema_docs`
  - Expectation: observability schema doc includes SLO threshold taxonomy and drift matrix.

## Success Metrics / Observable Signals

- Go/no-go checker rejects tampered SLO policy gate payloads deterministically.
- SLO gate taxonomy output remains stable and machine-readable.
- Docs contract tests fail closed on SLO threshold/gate marker drift.
