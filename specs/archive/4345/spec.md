# Spec: Issue #4345

Status: Implemented
Issue: #4345
Parent: #4341
Milestone: R27.32 Script-surface consolidation, documentation graduation, and architecture-navigability governance
Priority: P1

## Problem Statement

Current tests do not enforce dispatcher parity for the next wrapper family migration tranche or
stable fallback diagnostics for unknown wrapper failures.

## Scope

In scope:
- Add RED assertions for migrated task deep wrapper parity.
- Add RED assertions for deterministic unknown-wrapper fallback taxonomy markers.

Out of scope:
- Wrapper migration implementation (handled by #4346).

## Acceptance Criteria

AC-1:
Given task wrapper tests, when deep wrappers are not dispatcher-backed, then tests fail
non-flakily.

AC-2:
Given dispatcher matrix tests, when fallback markers drift, then tests fail with deterministic
messages.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Tests:
    - `bash scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
    - `bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
  - Expectation: fails RED before migration if deep wrappers are not dispatcher symlinks.

- C-02 (AC-2, Regression):
  - Test:
    - `bash scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
  - Expectation: fails RED before fallback marker implementation.

## Success Metrics / Observable Signals

- RED checks are deterministic and directly encode migration target behavior.
