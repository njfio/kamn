# Spec — #4289 Subtask: RED Tests for Failover Evidence Convergence

Status: Implemented
Priority: P1
Parent: #4283
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

Failover convergence behavior needs explicit failing tests for missing/tampered artifacts before implementation hardens the checker.

## Scope

In scope:
- RED tests for missing evidence links.
- RED tests for tampered convergence payloads.
- Deterministic repeated mismatch-order tests.

Out of scope:
- Production checker logic changes beyond what tests require.

## Acceptance Criteria

AC-1: Tests fail when required evidence-link markers are absent.

AC-2: Tests fail when convergence payloads are tampered.

AC-3: Repeated mismatch checks preserve deterministic reason ordering.

## Conformance Cases

- C-01 (AC-1, Regression): missing `report_file` linkage fails closed.
- C-02 (AC-2, Regression): tampered reason payload fails closed.
- C-03 (AC-3, Regression): repeated tamper checks emit stable reason ordering.

## Success Signals

- RED tests meaningfully fail prior to implementation.
- Regression suite captures artifact-link and tamper drift.
