# Spec — #4243 Subtask: Implement Replay Taxonomy Enforcement and Runbook Marker Parity Checks

Status: Reviewed
Priority: P1
Parent: #4237
Milestone: R27.25 Persistent journal replay and checkpoint-integrity governance

## Problem Statement

The sqlite crash-recovery checker does not enforce replay-idempotency taxonomy mapping/runbook
parity contracts end-to-end, allowing drift risk between emitted markers and operator runbook
declarations.

## Scope

In scope:
- Implement replay taxonomy/runbook constants and markers in run-lane and policy outputs.
- Add runbook-file parity validation in policy checker.
- Emit deterministic fail-closed reasons for replay taxonomy drift and runbook divergence.
- Wire contract-lane forwarding and docs updates.

Out of scope:
- Broader deployment workflow redesign.
- Non-sqlite checker migrations.

## Acceptance Criteria

AC-1: Policy checker validates replay taxonomy marker set and runbook parity markers.
AC-2: Missing/tampered runbook markers trigger `runbook_marker_parity_mismatch`.
AC-3: Replay taxonomy drift triggers deterministic replay taxonomy drift reason.
AC-4: Docs and docs tests enforce parity marker declaration in deploy/release runbooks.

## Conformance Cases

- C-01 (Functional): GO path emits replay taxonomy/runbook parity markers with verified status.
- C-02 (Regression): tampered replay mapping marker fails closed with deterministic drift reason.
- C-03 (Regression): runbook marker divergence fails closed with deterministic parity reason.
- C-04 (Integration): docs tests pass with new marker declarations in deploy/release docs.

## Success Signals

- New checker/runbook parity markers are present and validated in runtime policy and docs tests.
- Drift/divergence tamper fixtures fail closed with deterministic reason codes.
