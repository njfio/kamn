# Spec: Issue #4439

Status: Reviewed
Issue: #4439
Parent: #4433
Milestone: R27.38 SDK-client readiness, deployment packaging, and live-validation governance
Priority: P1

## Problem Statement

Packaging contract tests need explicit RED coverage for compose-manifest-config drift and invalid
artifact acceptance paths. Without those RED tests, fail-closed behavior can regress silently.

## Scope

In scope:
- RED test additions for compose drift, manifest drift, and packaging marker drift.
- Deterministic failure marker assertions for invalid packaging acceptance scenarios.

Out of scope:
- New packaging feature implementation outside test scaffolding.

## Acceptance Criteria

AC-1:
Given compose-topology contract lane tests, when packaging taxonomy/evidence markers are absent,
then tests fail deterministically.

AC-2:
Given compose-topology policy tests, when report taxonomy/reason markers are tampered, then tests
fail closed with deterministic reason expectations.

AC-3:
Given invalid compose/manifest/config fixtures, when contract checks run, then tests assert explicit
fail-closed reason markers.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
  - Expectation: fails RED when required packaging taxonomy/evidence markers are missing.

- C-02 (AC-2, Integration/Conformance):
  - Test: `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
  - Expectation: tampered taxonomy/reason CSV markers are rejected deterministically.

- C-03 (AC-3, Regression):
  - Tests:
    - `bash scripts/deploy/test_validate_compose_topology_contract_lane.sh`
    - `bash scripts/deploy/test_check_compose_topology_contract_policy.sh`
  - Expectation: invalid artifact acceptance paths fail with explicit reason-code markers.

## Success Metrics / Observable Signals

- RED assertions reproduce deterministic failures before implementation.
- Drift scenarios remain explicitly pinned and fail closed.
