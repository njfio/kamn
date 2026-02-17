# Spec — Issue #4810

- Title: Story: enforce permanent CI and process controls for shell-surface containment
- Parent: Parent epic: #4806
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Institutionalize fail-closed controls so future work cannot grow shell surface beyond bounded budgets and long-run shell<Rust target trajectory.

## Problem Statement

Without process and CI enforcement, reduction work can be undone by future feature churn and ad-hoc script additions.

## Scope

In scope:
- CI ratio/budget checks with deterministic reason taxonomy
- issue/PR process contracts that require shell-surface impact accounting
- ongoing telemetry and trend reporting for governance review

Out of scope:
- manual, ad-hoc spreadsheet governance outside repository automation
- non-deterministic policy gates

## Acceptance Criteria

- AC-1: CI fast gate blocks shell-surface regressions using deterministic reason outputs.
- AC-2: Issue/PR templates require shell LOC impact and mitigation sections for script-heavy changes.
- AC-3: Ongoing telemetry proves shell LOC trajectory toward and below Rust LOC with explicit thresholds.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence and fail-closed reasons.
- C-02: verify AC-2 with deterministic pass/fail evidence and fail-closed reasons.
- C-03: verify AC-3 with deterministic pass/fail evidence and fail-closed reasons.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Shell-surface reduction or containment impact is explicitly measurable for this scope.
