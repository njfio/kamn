# Spec - Issue #3646

- Title: Task: fix flaky root cause and remove quarantine from main test path
- Parent: #3631
- Milestone: R26.3 Reliability and cost governance (flake + script budget)
- Status: Implemented
- Priority: P0

## Problem Statement

Mainline reliability requires deterministic flaky-path stabilization and quarantine metadata cleanup.

## Objective

Close flaky root-cause and quarantine cleanup task by consolidating recurrence guards, metadata policy checks, and websocket CI-smoke stability evidence.

## Scope

In scope:
- Root-cause recurrence guard coverage (`#3797`).
- Quarantine/metadata cleanup governance coverage (`#3802`).
- Websocket CI-smoke stabilization coverage (`#3910`).
- Task lifecycle closure artifacts.

Out of scope:
- Unrelated test refactors.

## Acceptance Criteria

- AC-1: Previously flaky path remains deterministic under repeated checks.
- AC-2: Quarantine/ignore metadata cleanup policy remains fail closed on drift.
- AC-3: Closure evidence remains deterministic and auditable.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_check_websocket_session_ci_smoke_convergence.sh` and `bash scripts/ci/test_run_cargo_test_with_quarantine.sh` pass.
- C-02 (AC-2): `bash scripts/ci/test_check_flaky_registry.sh`, `bash scripts/ci/test_check_ignored_test_inventory_drift.sh`, and `bash scripts/ci/test_check_ignored_test_inventory_metadata_policy.sh` pass.
- C-03 (AC-3): all suites above pass in closure verification.

## Success Metrics

- Root-cause stability and quarantine cleanup governance remain deterministic in CI-compatible lanes.
