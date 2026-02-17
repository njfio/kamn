# Spec — #4261 Subtask: CI Smoke Checker for Partition-Finality Drift and Heavy-Lane Exclusion

Status: Reviewed
Priority: P1
Parent: #4254
Milestone: R27.26 Multi-node partition-healing and finality-convergence governance

## Problem Statement

No dedicated composite checker currently validates partition-finality CI smoke composition and
heavy-lane exclusion under one deterministic taxonomy.

## Scope

In scope:
- Add composite checker + red/green test harness.
- Validate fast-mode composition, heavy-lane leakage guards, and budget bound.

Out of scope:
- Runtime lane behavior changes.

## Acceptance Criteria

AC-1: Baseline checker run passes with deterministic markers.

AC-2: Leaked heavy-lane commands fail checker with deterministic reasons.

AC-3: Budget overflow fails checker with deterministic reason.

## Conformance Cases

- C-01 (AC-1): baseline run returns `status=pass` and `reason_codes_value=none`.
- C-02 (AC-2): leaked heavy command in workflow/fast-mode block fails with exclusion reason.
- C-03 (AC-3): `--max-seconds > 120` fails with budget-exceeded reason.
