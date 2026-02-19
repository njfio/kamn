# Spec — #4285 Subtask: Add Red Tests for Live-Node Drift Marker Mismatch Rejection Behavior

Status: Implemented
Priority: P1
Parent: #4281
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

The failover preflight contract lane lacks explicit red tests proving the checker rejects missing live-node drift markers and mismatch drift states deterministically.

## Scope

In scope:
- Red tests for missing-marker rejection.
- Red tests for drift mismatch rejection behavior.
- Red tests for deterministic repeated mismatch reason output.

Out of scope:
- Drift model redesign.

## Acceptance Criteria

AC-1: Tests fail when required drift markers are missing.

AC-2: Tests fail when live-node drift parity marker is drifted.

AC-3: Regression test verifies deterministic mismatch reason output stability.

## Conformance Cases

- C-01 (AC-1, Regression): missing `live_node_drift_parity_status` marker fails with deterministic required-field reason.
- C-02 (AC-2, Regression): drifted `live_node_drift_parity_status` fails with `live_node_drift_marker_parity_mismatch`.
- C-03 (AC-3, Regression): repeated checks over identical drift report return stable reason ordering and content.

## Success Signals

- New tests fail before checker policy mode exists.
- Tests pass after checker policy mode implementation.
