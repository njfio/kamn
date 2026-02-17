# Spec: #4367 Deployment Gate Reason Taxonomy + CI/Local Boundary Enforcement

Status: Reviewed (agent-authored; human review requested in PR)

## Problem Statement

Milestone aggregate go/no-go convergence should encode deterministic deployment-safety taxonomy behavior for key-policy rotation and CI/local-heavy rollout boundaries.

## Scope

In scope:
- Enforce rotation taxonomy and boundary marker contracts in milestone aggregate checker.
- Emit deterministic fail-closed reason mappings for drift.

Out of scope:
- New runtime deployment lanes.

## Acceptance Criteria

AC-1 Rotation taxonomy marker mismatches are rejected with deterministic reason mapping.
AC-2 CI-smoke/local-heavy boundary marker mismatches are rejected with deterministic reason mapping.
AC-3 Valid aggregate evidence remains GO with stable marker outputs.

## Conformance Cases

- C-01 (AC-1): required rotation taxonomy markers present on pass path.
- C-02 (AC-1): drifted rotation taxonomy marker maps to stable fail-closed reason.
- C-03 (AC-2): drifted boundary marker maps to stable fail-closed reason.
- C-04 (AC-3): unchanged valid artifacts keep deterministic GO.

## Success Metrics

- Deterministic GO/NO-GO paths are preserved with explicit reason markers.
