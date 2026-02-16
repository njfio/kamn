# Spec: Issue #4339

Status: Reviewed
Issue: #4339
Parent: #4338
Milestone: R27.32 Script-surface consolidation, documentation graduation, and architecture-navigability governance
Priority: P1

## Problem Statement

Wrapper-heavy lane families still include non-dispatch wrappers that increase maintenance burden and
raise drift risk between lane entrypoints and shared execution modules.

## Scope

In scope:
- Migrate a selected wrapper family to manifest-driven non-Kolme dispatcher paths.
- Add deterministic parity checks for migrated wrappers.
- Add deterministic dispatcher fallback reason markers for unknown wrappers.

Out of scope:
- Full repository-wide wrapper retirement.
- Script budget threshold policy work tracked in #4342.

## Acceptance Criteria

AC-1:
Given selected wrapper-family deep lanes, when migration is complete, then wrappers execute through
`run_non_kolme_contract_lane_dispatch.sh` with manifest-backed resolution.

AC-2:
Given wrapper/manifest dispatch parity checks, when wrapper wiring drifts, then tests fail closed
with deterministic diagnostics.

AC-3:
Given unknown wrapper dispatch attempts, when dispatcher fails closed, then deterministic fallback
reason taxonomy markers are emitted.

## Conformance Cases

- C-01 (AC-1, Integration):
  - Tests:
    - `bash scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
    - `bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
  - Expectation: task deep-lane wrappers are dispatcher symlinks resolving to deep manifests.

- C-02 (AC-2, Functional/Regression):
  - Test:
    - `bash scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
  - Expectation: migrated wrappers resolve existing manifests and unknown wrappers fail closed.

- C-03 (AC-3, Functional):
  - Test:
    - `bash scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
  - Expectation: unknown-wrapper fallback emits deterministic taxonomy/reason-code markers.

## Success Metrics / Observable Signals

- Selected wrapper family no longer relies on bespoke deep wrapper shell logic.
- Dispatcher fallback outputs are deterministic and contract-test asserted.
