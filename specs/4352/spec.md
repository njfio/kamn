# Spec — #4352 Subtask: Implement Rustdoc Publication + Ratio Governance Enforcement

Status: Reviewed
Priority: P1
Parent: #4344

## Problem Statement

Rustdoc publication checker needs deterministic ratio governance outputs so docs-contract growth cannot outpace behavioral validation coverage in this contract lane.

## Acceptance Criteria

AC-1: Report schema includes deterministic docs/behavioral ratio fields.

AC-2: Policy checker validates ratio fields and fails closed when threshold exceeded.

AC-3: Checker emits deterministic reason marker for ratio threshold exceedance.

## Conformance Cases

- C-01 (AC-1): lane report contains ratio fields and status marker.
- C-02 (AC-2): policy checker rejects over-threshold ratio reports.
- C-03 (AC-3): checker error output includes ratio reason marker.
