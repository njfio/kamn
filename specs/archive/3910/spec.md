# Spec - Issue #3910

- Title: Subtask: stabilize websocket missing-upgrade-header regression test across CI
- Parent: #3646
- Milestone: R26.3 Reliability and cost governance (flake + script budget)
- Status: Implemented
- Priority: P0

## Problem Statement

Websocket missing-upgrade-header regression behavior must be deterministic across CI so merge gates are not destabilized by intermittent failures.

## Objective

Close websocket anti-flake stabilization with deterministic CI-smoke convergence and quarantine-lane validation coverage.

## Scope

In scope:
- CI-smoke websocket convergence checker coverage.
- Quarantine-lane repeated-run behavior validation.
- Subtask lifecycle closure evidence.

Out of scope:
- Websocket API redesign.

## Acceptance Criteria

- AC-1: Websocket regression behavior is deterministic in CI-smoke checker coverage.
- AC-2: Repeated anti-flake lane behavior remains stable and auditable.
- AC-3: Conformance evidence is deterministic and green.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_check_websocket_session_ci_smoke_convergence.sh` passes.
- C-02 (AC-2): `bash scripts/ci/test_run_cargo_test_with_quarantine.sh` passes.
- C-03 (AC-3): both suites above pass in closure verification.

## Success Metrics

- Websocket anti-flake convergence checks remain deterministic and fail closed on drift.
