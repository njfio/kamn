# Spec: Issue #4472

Status: Reviewed
Issue: #4472
Parent: #4464
Milestone: R27.40 Compliance audit-trail integrity, SLO-governance, and incident-readiness contracts
Priority: P1

## Problem Statement

Incident go/no-go lanes require deterministic boundary reason taxonomy and explicit CI smoke versus
local-heavy governance to keep promotion evidence auditable and CI cost bounded.

## Scope

In scope:
- Implement incident boundary reason taxonomy/version marker emission in go/no-go contract lane.
- Enforce CI smoke max-seconds boundary and local-heavy opt-in guard for incident deep lane.
- Update CI strategy docs and docs tests for incident boundary governance matrix.

Out of scope:
- Fast-gate enablement for deep incident drills.

## Acceptance Criteria

AC-1:
Given incident go/no-go contract lane execution, when successful under bounded CI smoke settings,
then deterministic boundary taxonomy/version/reason markers are emitted.

AC-2:
Given CI smoke/local-heavy boundary violations, when lane executes, then fail-closed reason codes
are deterministic and auditable.

AC-3:
Given docs checks run, when incident boundary docs drift, then CI docs tests fail closed.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - Expectation: bounded lane emits deterministic incident boundary governance markers.

- C-02 (AC-2, Functional/Regression):
  - Test: `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
  - Expectation: boundary violations produce deterministic fail-closed reason codes.

- C-03 (AC-3, Docs):
  - Test: `cargo test -p kamn-core --test ci_strategy_docs`
  - Expectation: CI strategy includes incident boundary taxonomy + ci/local matrix references.

## Success Metrics / Observable Signals

- Incident go/no-go lane boundaries are deterministic and bounded.
- CI strategy docs pin incident boundary governance and fail-closed markers.
