# Spec: Issue #4463

Status: Implemented
Issue: #4463
Parent: #4460
Milestone: R27.40 Compliance audit-trail integrity, SLO-governance, and incident-readiness contracts
Priority: P1

## Problem Statement

Release go/no-go evidence lacks a dedicated incident-readiness bundle convergence gate. Without
explicit schema and freshness validation for incident readiness artifacts, stale or tampered
readiness evidence can bypass promotion safety.

## Scope

In scope:
- Add optional incident-readiness report ingestion to go/no-go generator/checker contract flow.
- Fail closed on missing, invalid, stale, mismatched, or non-GO incident-readiness evidence.
- Emit deterministic incident-readiness reason taxonomy and normalized reason markers.
- Add tests and docs updates for incident-readiness bundle schema convergence.

Out of scope:
- Incident platform orchestration automation.
- New runtime incident telemetry backend.

## Acceptance Criteria

AC-1:
Given go/no-go generation with incident-readiness evidence enabled, when incident-readiness
artifacts are missing, stale, invalid, or non-GO, then final decision fails closed deterministically.

AC-2:
Given incident-readiness evidence is enabled, when generator/checker executes, then deterministic
incident-readiness taxonomy and reason output markers are emitted and audited.

AC-3:
Given tampered incident-readiness gate payloads, when checker validates bundle, then checker fails
closed with deterministic convergence mismatch.

AC-4:
Given incident readiness docs contract tests run, when docs are validated, then incident-readiness
bundle schema, taxonomy, and mismatch/tamper failure references are present and drift-protected.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: missing/stale/invalid/non-GO incident readiness artifacts produce deterministic
    NO-GO reason outputs.

- C-02 (AC-2, Functional):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: deterministic incident readiness gate taxonomy/csv/value markers for GO and NO-GO.

- C-03 (AC-3, Regression/Conformance):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: tampered `incident_readiness_gate` payload fails with deterministic convergence
    mismatch.

- C-04 (AC-3, Integration):
  - Test: `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - Expectation: contract lane validates incident readiness gate convergence argument wiring.

- C-05 (AC-4, Docs):
  - Test: `cargo test -p kamn-core --test incident_readiness_docs`
  - Expectation: incident readiness docs contain go/no-go convergence gate commands and markers.

## Success Metrics / Observable Signals

- Go/no-go checker rejects tampered incident-readiness gate payloads deterministically.
- Incident-readiness gate reason outputs remain stable and machine-readable.
- Incident readiness docs fail closed on schema/taxonomy marker drift.
