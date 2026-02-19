# Spec: Issue #4442

Status: Implemented
Issue: #4442
Parent: #4434
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

Live go/no-go gating needs a deterministic reason taxonomy surface and explicit CI smoke/local-heavy
boundary governance so promotion decisions remain auditable and CI cost stays bounded.

## Scope

In scope:
- Implement deterministic live-gate reason taxonomy/version/reason markers.
- Enforce CI smoke versus local-heavy boundary controls in go/no-go lane wrappers.
- Update CI strategy and release checklist docs plus docs-contract tests.

Out of scope:
- Always-on deep live-validation in CI fast-gate.

## Acceptance Criteria

AC-1:
Given bounded live go/no-go contract lane execution, when lane succeeds, then deterministic
live-gate taxonomy markers are emitted.

AC-2:
Given CI smoke/local-heavy boundary violations, when lane executes, then fail-closed reason codes
are deterministic and auditable.

AC-3:
Given docs checks run, when live-go/no-go boundary or taxonomy markers drift, then docs tests fail
closed.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - Expectation: contract lane emits deterministic live-go/no-go taxonomy and boundary markers.

- C-02 (AC-2, Functional/Regression):
  - Tests:
    - `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
    - `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
  - Expectation: overflow/opt-in violations and live evidence mismatches fail closed deterministically.

- C-03 (AC-3, Docs):
  - Tests:
    - `cargo test -p kamn-core --test ci_strategy_docs`
    - `cargo test -p kamn-core --test release_gonogo_checklist_docs`
  - Expectation: docs retain live-go/no-go boundary and taxonomy markers.

## Success Metrics / Observable Signals

- Live-go/no-go boundary contracts are deterministic, bounded, and documented.
