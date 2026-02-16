# Spec: Issue #4440

Status: Implemented
Issue: #4440
Parent: #4433
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

Deployment packaging validation detects drift, but failure reason taxonomy and normalized evidence
outputs are not fully versioned and deterministic for policy auditing.

## Scope

In scope:
- Deterministic packaging reason taxonomy version and reason-code CSV surfaces.
- Stable evidence output markers in lane summary and policy checker reports.
- Deterministic mapping between failed checks and emitted reason codes.

Out of scope:
- Non-packaging deployment governance areas.

## Acceptance Criteria

AC-1:
Given compose-topology lane success, when reports are generated, then packaging taxonomy markers
and evidence status markers are emitted in stdout and JSON.

AC-2:
Given policy checker validation, when summary taxonomy/reason/evidence markers drift, then checker
fails with deterministic reason codes.

AC-3:
Given docs and test contracts, when taxonomy marker surface changes unexpectedly, then contract
checks fail closed.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
  - Expectation: lane emits deterministic taxonomy/version/reason/evidence markers.

- C-02 (AC-2, Conformance/Integration):
  - Test: `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
  - Expectation: policy checker deterministic reason list includes taxonomy/evidence mismatch codes.

- C-03 (AC-3, Docs/Regression):
  - Tests:
    - `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
    - `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
  - Expectation: docs references and reason taxonomy markers remain pinned.

## Success Metrics / Observable Signals

- Packaging policy decisions include deterministic taxonomy and reason-code evidence.
- Drift classifications are machine-readable and stable across runs.
