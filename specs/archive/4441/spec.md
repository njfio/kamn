# Spec: Issue #4441

Status: Implemented
Issue: #4441
Parent: #4434
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

Live-validation evidence bundle contracts need explicit RED coverage for mismatch, tamper, and
partial-evidence acceptance paths so deterministic fail-closed behavior cannot regress unnoticed.

## Scope

In scope:
- Add RED tests for live milestone evidence tamper/mismatch and partial-evidence acceptance.
- Encode deterministic expected fail-closed error and reason-code surfaces.

Out of scope:
- New live lane orchestration features.

## Acceptance Criteria

AC-1:
Given tampered live milestone evidence bundles, when policy checker runs, then tests fail if
tampering is accepted.

AC-2:
Given partial live evidence inputs, when bundle generation/checking runs, then tests fail if final
decision does not fail closed with deterministic reason signals.

AC-3:
Given regression suite execution, when live mismatch/tamper paths run, then deterministic fail
closed behavior remains stable.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: tampered milestone lineage payload is rejected deterministically.

- C-02 (AC-2, Functional):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: partial live evidence path yields deterministic NO-GO reason output.

- C-03 (AC-3, Regression):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: mismatch/tamper fail-closed behavior remains deterministic.

## Success Metrics / Observable Signals

- RED tests encode live mismatch/tamper/partial acceptance failure expectations before GREEN changes.
