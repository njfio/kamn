# Spec — #4288 Subtask: Implement Drift Taxonomy Enforcement and Runbook Marker Parity Contract Checks

Status: Implemented
Priority: P1
Parent: #4282
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

Failover promotion governance requires deterministic checker enforcement for taxonomy/runbook parity; otherwise marker drift can bypass remediation safeguards.

## Scope

In scope:
- Taxonomy mapping validation logic.
- Runbook marker parity checks.
- Deterministic fail-closed reason mapping and policy outputs.

Out of scope:
- Incident tooling redesign.

## Acceptance Criteria

AC-1: Drift taxonomy mapping remains deterministic.

AC-2: Runbook parity checks fail closed on marker drift.

AC-3: Contract lane tests preserve checker/runbook alignment.

## Conformance Cases

- C-01 (AC-1, Functional): checker accepts valid taxonomy/runbook marker set.
- C-02 (AC-1, Regression): drifted taxonomy status fails with deterministic taxonomy-drift reason.
- C-03 (AC-2, Regression): runbook marker divergence fails with deterministic runbook-parity reason.
- C-04 (AC-2, Regression): taxonomy version/csv mismatch fails with deterministic mismatch reasons.
- C-05 (AC-3, Functional): policy output remains deterministic and machine-readable across repeated checks.

## Success Signals

- Taxonomy and runbook marker drift reject deterministically.
- Policy outputs and docs remain synchronized.
