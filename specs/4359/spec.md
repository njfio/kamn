# Spec: #4359 Deployment Safety Gate Convergence for Key-Policy Rotation and Rollout Evidence

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

Milestone-level go/no-go evidence aggregation validates artifact presence and high-level decisions, but does not enforce deterministic convergence on rotation-preflight taxonomy outputs or rollout boundary markers (CI smoke vs local-heavy drill scope). This leaves a gap where stale or drifted deployment-safety evidence can pass as structurally valid.

## Scope

In scope:
- `scripts/deploy/gonogo_evidence_contract.py` milestone review convergence checks for:
  - deployment preflight policy rotation taxonomy markers
  - go/no-go gate CI smoke/local-heavy boundary markers
- RED/green tests for missing/drifted deployment-safety evidence across key-policy/rotation and rollout boundaries.
- docs update for CI boundary/convergence markers.

Out of scope:
- Runtime go/no-go lane redesign.
- Secret-store platform migration.

## Acceptance Criteria

AC-1 Deployment gate fails closed when rotation-preflight taxonomy evidence is missing/drifted.
AC-2 Deployment gate fails closed when go/no-go boundary markers drift from CI-smoke/local-heavy contract.
AC-3 Reason outputs remain deterministic and auditable for pass/fail paths.
AC-4 Integration and regression tests cover deployment safety convergence with stable marker expectations.

## Conformance Cases

- C-01 (AC-1): milestone aggregate GO fixture requires deployment preflight policy rotation taxonomy version/codes/value markers.
- C-02 (AC-1): drifted rotation taxonomy marker yields deterministic `NO-GO` and a stable reason code.
- C-03 (AC-2): drifted go/no-go boundary marker set yields deterministic `NO-GO` and a stable reason code.
- C-04 (AC-3/AC-4): valid milestone aggregate artifacts preserve deterministic GO path and policy parity.

## Success Metrics

- Red tests fail before implementation and pass after implementation.
- Milestone aggregate evidence checker remains deterministic under drift fixtures.
- CI strategy docs include deployment safety checker boundary/convergence markers.
