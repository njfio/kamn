# Spec: Issue #4461

Status: Implemented
Issue: #4461
Parent: #4459
Milestone: R27.40 Compliance audit-trail integrity, SLO-governance, and incident-readiness contracts
Priority: P1

## Problem Statement

Release go/no-go evidence currently validates base, milestone, and TLS gates, but does not enforce
an explicit audit-trail integrity convergence gate. Without that gate, stale/tampered audit-policy
artifacts can bypass deterministic release governance.

## Scope

In scope:
- Add optional audit-integrity gate ingestion to go/no-go generator/checker contract.
- Fail closed on missing/invalid/stale/tampered audit-policy evidence.
- Emit deterministic audit-integrity reason taxonomy and normalized reason-code markers.
- Add tests and docs updates for audit-trail contract controls.

Out of scope:
- New external compliance/SIEM integrations.
- Runtime storage engine redesign.

## Acceptance Criteria

AC-1:
Given go/no-go generation with audit evidence enabled, when the audit policy report is missing,
invalid, stale, or non-passing, then final decision fails closed deterministically.

AC-2:
Given audit evidence is enabled, when generator/checker executes, then deterministic
audit-integrity taxonomy markers and normalized reason-code outputs are emitted.

AC-3:
Given a tampered bundle, when checker validates audit-integrity gate payloads, then checker fails
closed on deterministic convergence mismatch.

AC-4:
Given docs contracts run, when release and ops docs are validated, then audit-trail go/no-go policy
controls and taxonomy references are present and drift-protected.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: audit missing/invalid/stale/non-pass inputs map to deterministic NO-GO.

- C-02 (AC-2, Unit/Functional):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: deterministic `audit_integrity_reason_taxonomy_version`,
    `audit_integrity_reason_codes_csv`, and `audit_integrity_reason_codes_value` markers.

- C-03 (AC-3, Regression/Conformance):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: tampered audit gate payload fails with explicit convergence mismatch.

- C-04 (AC-3, Integration):
  - Test: `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - Expectation: contract lane exercises audit-integrity gate and preserves deterministic GO/NO-GO.

- C-05 (AC-4, Docs):
  - Test: `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - Expectation: release checklist includes audit-integrity gate markers and commands.

- C-06 (AC-4, Docs):
  - Test: `cargo test -p kamn-core --test service_api_ops_configuration_docs`
  - Expectation: ops config doc includes audit-tamper fail-closed controls.

## Success Metrics / Observable Signals

- Go/no-go checker rejects tampered audit-integrity gate payloads deterministically.
- Audit-integrity reason taxonomy output is stable and machine-readable.
- Docs contract tests fail closed on audit policy marker drift.
