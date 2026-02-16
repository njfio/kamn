# Spec: Issue #4433

Status: Reviewed
Issue: #4433
Parent: #4430
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

Deployment packaging checks currently validate compose/manifests/config artifacts, but the contract
surface for deterministic packaging reason taxonomy and evidence normalization is incomplete. Drift
can be detected, yet the emitted lane evidence is not explicit enough for stable audit and promotion
decisions.

## Scope

In scope:
- Compose/manifest/config invariant checks for deployment packaging lanes.
- Deterministic packaging reason taxonomy version and reason-code CSV surfaces.
- Deployment evidence outputs that remain stable on pass/fail paths.
- Docs updates for packaging contract markers and governance boundaries.

Out of scope:
- Full cluster rollout automation.
- New deployment orchestrators or runtime protocols.

## Acceptance Criteria

AC-1:
Given deployment packaging contract lane execution, when compose/manifest/config invariants drift,
then the lane fails closed with deterministic reason-code markers.

AC-2:
Given packaging policy evaluation, when lane summary markers or taxonomy fields drift, then policy
checker returns NO-GO with deterministic failure reason codes.

AC-3:
Given successful packaging lane execution, when reports are emitted, then deterministic packaging
taxonomy version/reason CSV/evidence fields are present in stdout and JSON outputs.

AC-4:
Given deploy/CI docs checks, when packaging contract taxonomy or reason markers drift, then tests
fail closed.

## Conformance Cases

- C-01 (AC-1, Functional/Regression):
  - Test: `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
  - Expectation: compose/manifest/config tamper scenarios fail with deterministic markers.

- C-02 (AC-2, Conformance/Integration):
  - Test: `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
  - Expectation: policy checker rejects taxonomy/report drift with deterministic failure codes.

- C-03 (AC-3, Functional):
  - Tests:
    - `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
    - `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
  - Expectation: taxonomy version/reason CSV/evidence markers are emitted and stable.

- C-04 (AC-4, Docs):
  - Tests:
    - `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
    - `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
  - Expectation: docs include packaging taxonomy and fail-closed reason markers.

## Success Metrics / Observable Signals

- Packaging drift is fail-closed with explicit deterministic reason codes.
- Policy output remains stable and machine-readable for release governance.
- Docs and test contracts prevent packaging taxonomy/evidence drift.
