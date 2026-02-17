# Spec — #4282 Task: Enforce Drift Taxonomy and Runbook Marker Parity Checks

Status: Implemented
Priority: P1
Parent: #4279
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

Deterministic failover drift governance requires parity between checker-enforced taxonomy markers and runbook-declared marker contracts. Drift between checker output and runbook declarations can mislead promotion decisions.

## Scope

In scope:
- Drift taxonomy mapping enforcement for failover preflight evidence.
- Runbook marker parity validation in policy checking.
- Red/regression tests for taxonomy drift and runbook marker divergence.
- Docs updates for release go/no-go taxonomy parity references.

Out of scope:
- Runbook process redesign.
- Incident tooling redesign.

## Acceptance Criteria

AC-1: Drift taxonomy mapping remains deterministic and checker-validated.

AC-2: Runbook marker parity checks fail closed on drift or missing declarations.

AC-3: Regression tests preserve taxonomy/runbook checker alignment.

AC-4: Release go/no-go checklist and deploy runbook references remain synchronized with checker markers.

## Conformance Cases

- C-01 (AC-1, Functional): valid preflight report passes taxonomy and runbook parity checks.
- C-02 (AC-1, Regression): drifted taxonomy marker fails with deterministic taxonomy-drift reason.
- C-03 (AC-2, Regression): runbook missing required marker declaration fails with deterministic runbook-parity reason.
- C-04 (AC-2, Regression): drifted taxonomy version/csv markers fail with deterministic mismatch reasons.
- C-05 (AC-3, Regression): repeated checks over identical drift fixture preserve deterministic reason ordering.
- C-06 (AC-4, Conformance): docs contract tests assert release checklist and deploy runbook marker parity strings.

## Success Signals

- Taxonomy and runbook marker drift fail closed deterministically.
- Policy checker and docs markers remain synchronized.
- Regression coverage guards future taxonomy/runbook divergence.
