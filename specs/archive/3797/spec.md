# Spec - Issue #3797

- Title: Subtask: land deterministic flaky root-cause fix with stress regression coverage
- Parent: #3646
- Milestone: R26.3 Reliability and cost governance (flake + script budget)
- Status: Implemented
- Priority: P0

## Problem Statement

Root-cause flaky behavior must be eliminated with deterministic stress/regression validation rather than quarantine masking.

## Objective

Close root-cause stability evidence using websocket CI-smoke convergence and repeated quarantine-lane validation.

## Scope

In scope:
- Websocket anti-flake convergence coverage.
- Quarantine-lane repeated-run stability coverage.
- Subtask lifecycle closure artifacts.

Out of scope:
- Unrelated test refactors.

## Acceptance Criteria

- AC-1: Previously flaky path remains stable under deterministic repeated checks.
- AC-2: Regression guards fail closed on recurrence.
- AC-3: Conformance evidence is deterministic and green.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_check_websocket_session_ci_smoke_convergence.sh` passes.
- C-02 (AC-2): `bash scripts/ci/test_run_cargo_test_with_quarantine.sh` passes.
- C-03 (AC-3): both suites above pass in closure verification.

## Success Metrics

- Root-cause anti-flake stability evidence remains deterministic across CI-compatible regression lanes.
