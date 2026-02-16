# Spec: Issue #4434

Status: Reviewed
Issue: #4434
Parent: #4430
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

Go/no-go live-validation evidence currently validates aggregate milestone lineage, but deterministic
live-gate taxonomy and explicit CI smoke versus local-heavy boundary governance are underspecified.
Without that contract surface, mismatch/tamper drift can become harder to audit and deep-lane
execution can leak cost into fast-gate paths.

## Scope

In scope:
- Add RED coverage for live-validation mismatch, tamper, and partial-evidence acceptance paths.
- Emit deterministic live-gate reason taxonomy markers and reason-code surfaces.
- Enforce explicit CI smoke/local-heavy boundary controls for live go/no-go lane execution.
- Update docs and docs-contract tests for live go/no-go boundary governance drift protection.

Out of scope:
- Always-on deep live-validation drills in CI fast-gate.
- New external release orchestration systems.

## Acceptance Criteria

AC-1:
Given live-validation milestone evidence bundle generation/checking, when linked live evidence is
mismatched or tampered, then policy fails closed with deterministic live-gate mismatch signals.

AC-2:
Given live-validation bundle generation, when live evidence is partial/incomplete, then final
decision is NO-GO with deterministic reason codes.

AC-3:
Given go/no-go contract lane execution under bounded CI smoke mode, when lane succeeds, then
deterministic live-gate reason taxonomy and CI/local boundary markers are emitted.

AC-4:
Given CI strategy and release go/no-go checklist docs tests run, when live-gate taxonomy or
boundary matrix markers drift, then tests fail closed.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: tampered live milestone lineage evidence is rejected by policy checker.

- C-02 (AC-2, Functional/Regression):
  - Test: `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: partial live-validation evidence forces deterministic NO-GO reason output.

- C-03 (AC-3, Integration):
  - Test: `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - Expectation: CI smoke/local-heavy boundary markers and deterministic live-gate taxonomy markers
    are emitted, with fail-closed overflow/opt-in signals.

- C-04 (AC-4, Docs):
  - Tests:
    - `cargo test -p kamn-core --test ci_strategy_docs`
    - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - Expectation: docs pin deterministic live go/no-go boundary and taxonomy markers.

## Success Metrics / Observable Signals

- Live go/no-go mismatch/tamper/partial-evidence acceptance remains fail-closed and deterministic.
- CI smoke/local-heavy boundaries remain explicit, bounded, and machine-readable.
- Docs contracts prevent taxonomy/boundary drift for live go/no-go governance.
