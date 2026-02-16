# Spec: Issue #4464

Status: Implemented
Issue: #4464
Parent: #4460
Milestone: R27.40 Compliance audit-trail integrity, SLO-governance, and incident-readiness contracts
Priority: P1

## Problem Statement

Incident go/no-go readiness now has convergence checks, but deploy-lane boundary governance remains
underspecified. CI smoke and local-heavy drill execution need deterministic, enforceable boundaries
so incident evidence validation stays auditable without expanding fast-gate cost.

## Scope

In scope:
- Enforce CI smoke boundary controls for go/no-go evidence contract lane execution.
- Enforce explicit local-only opt-in boundary for incident deep-lane drills.
- Emit deterministic incident boundary reason taxonomy markers and fail-closed reason codes.
- Update CI strategy docs and docs tests for boundary governance drift protection.

Out of scope:
- Always-on deep incident drills in CI fast-gate.
- Incident orchestration platform automation.

## Acceptance Criteria

AC-1:
Given go/no-go incident evidence contract lane execution, when CI smoke boundary parameters exceed
allowed limits, then lane fails closed with deterministic boundary reason code.

AC-2:
Given go/no-go incident deep-lane execution, when local-heavy opt-in is missing, then lane fails
closed with deterministic opt-in boundary reason code.

AC-3:
Given go/no-go contract lane execution in bounded mode, when lane succeeds, then deterministic
incident boundary reason taxonomy markers and boundary profiles are emitted.

AC-4:
Given CI strategy docs contract tests run, when incident go/no-go boundary docs drift, then tests
fail closed on missing taxonomy, reason-code, and boundary matrix markers.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - Expectation: CI smoke max-seconds overflow returns deterministic boundary failure marker.

- C-02 (AC-2, Functional/Conformance):
  - Test: `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - Expectation: deep lane without local-heavy opt-in fails with deterministic opt-in reason code.

- C-03 (AC-3, Integration):
  - Test: `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - Expectation: contract lane emits deterministic boundary taxonomy/version/reason markers.

- C-04 (AC-3, Regression):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: incident readiness partial-evidence convergence gap remains fail-closed.

- C-05 (AC-4, Docs):
  - Test: `cargo test -p kamn-core --test ci_strategy_docs`
  - Expectation: CI strategy doc includes incident go/no-go boundary matrix and failure taxonomy.

## Success Metrics / Observable Signals

- Incident go/no-go lanes enforce explicit CI smoke and local-heavy boundary guards.
- Boundary reason markers are deterministic and machine-readable.
- CI strategy docs are drift-protected for incident boundary governance rules.
