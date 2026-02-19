# Spec — #4286 Subtask: Implement Deterministic Drift Checker Outputs and Fail-Closed Reason Mapping

Status: Implemented
Priority: P1
Parent: #4281
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

Failover drift governance needs explicit checker logic that validates marker contracts and emits deterministic fail-closed reasons for promotion gating.

## Scope

In scope:
- Add policy-check mode for failover preflight reports.
- Enforce deterministic marker and taxonomy checks.
- Emit deterministic reason outputs and policy report JSON.

Out of scope:
- New lane executables.
- Failover simulation model redesign.

## Acceptance Criteria

AC-1: Policy checker validates required failover preflight markers deterministically.

AC-2: Drift mismatch/missing marker/taxonomy mismatch conditions fail closed with stable reasons.

AC-3: Checker outputs deterministic policy report schema and reason markers.

## Conformance Cases

- C-01 (AC-1, Functional): valid report + `ci-fast-gate=PASS` yields `status=ok`, `final_decision=GO`, and verified policy marker.
- C-02 (AC-2, Regression): live-node drift mismatch fails with `live_node_drift_marker_parity_mismatch`.
- C-03 (AC-2, Regression): missing required field fails with deterministic required-field reason.
- C-04 (AC-2, Regression): reason taxonomy version/csv drift fails with deterministic taxonomy mismatch reasons.
- C-05 (AC-3, Functional): policy report writes deterministic schema and reason-code projections.

## Success Signals

- Checker rejects all tampered drift reports deterministically.
- Policy outputs remain stable across repeated runs.
