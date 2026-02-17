# Spec — #4344 Task: Rustdoc Navigation Publication + Docs/Behavioral Ratio Governance

Status: Reviewed
Priority: P1
Parent: #4340
Milestone: R27.32 Script-surface consolidation, documentation graduation, and architecture-navigability governance

## Problem Statement

Rustdoc/navigation publication checks exist, but governance does not currently enforce a deterministic docs-contract-to-behavioral-test ratio contract for this lane family.

## Scope

In scope:
- RED tests for publication drift and docs-heavy ratio imbalance (#4351).
- Checker/report updates for deterministic ratio governance outputs (#4352).
- Docs updates describing the ratio governance marker contract.

Out of scope:
- Full CI-wide test taxonomy redesign.

## Acceptance Criteria

AC-1: Rustdoc publication checker remains fail-closed for artifact/navigation drift.

AC-2: Docs-vs-behavioral ratio policy is validated with deterministic markers and failure reason output.

AC-3: CI smoke path remains bounded and deterministic after governance updates.

AC-4: Task/subtask chain closes with spec-backed test evidence.

## Conformance Cases

- C-01 (AC-1): tampered rustdoc artifact report still fails policy.
- C-02 (AC-2): ratio imbalance scenario fails with deterministic reason marker.
- C-03 (AC-2): normal ratio scenario passes with deterministic ratio markers.
- C-04 (AC-3): ci-tools fast mode remains green.
