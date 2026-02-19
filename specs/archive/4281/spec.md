# Spec — #4281 Task: Implement Live-Node Drift Checker and Deterministic Mismatch Reason Mapping

Status: Implemented
Priority: P1
Parent: #4279
Milestone: R27.28 Live-node drift detection and failover-readiness governance

## Problem Statement

Failover preflight evidence must fail closed whenever live-node drift marker parity diverges from deterministic contracts. Missing markers and drifted marker values can otherwise pass through promotion gates.

## Scope

In scope:
- Deterministic checker validation for failover preflight drift markers.
- Stable fail-closed reason mapping for drift, stall, and CI/local budget boundary conditions.
- Red-test and regression coverage for missing marker and drift tamper cases.
- Ops docs updates for drift marker mismatch policy contracts.

Out of scope:
- New failover model design.
- Runtime transport redesign.

## Acceptance Criteria

AC-1: Checker validates required failover preflight drift markers deterministically.

AC-2: Marker mismatch and missing-marker cases fail closed with stable reason outputs.

AC-3: Regression tests cover live-node drift mismatch rejection behavior.

AC-4: `docs/ops/configuration.md` includes drift marker mismatch policy contracts.

## Conformance Cases

- C-01 (AC-1, Functional): policy checker returns `status=ok` for valid preflight report with expected marker set.
- C-02 (AC-2, Regression): report missing `live_node_drift_parity_status` fails with deterministic required-field reason.
- C-03 (AC-2, Regression): drifted `live_node_drift_parity_status` fails with `live_node_drift_marker_parity_mismatch`.
- C-04 (AC-2, Regression): drifted reason taxonomy/version markers fail with deterministic taxonomy mismatch reasons.
- C-05 (AC-3, Regression): repeated policy checks for identical drift report produce deterministic identical reason outputs.
- C-06 (AC-4, Conformance): docs contract test asserts failover drift policy markers and reason taxonomy strings.

## Success Signals

- Drift mismatch and missing-marker reports are rejected deterministically.
- Reason-code mapping remains stable across repeated checks.
- Docs and docs-contract tests remain synchronized with checker behavior.
