# Spec: Issue #4346

Status: Implemented
Issue: #4346
Parent: #4341
Milestone: R27.32 Script-surface consolidation, documentation graduation, and architecture-navigability governance
Priority: P1

## Problem Statement

Selected task deep wrappers still use direct shell logic and non-Kolme dispatcher unknown-wrapper
failures do not publish deterministic fallback taxonomy markers for policy tooling.

## Scope

In scope:
- Add task deep-lane manifests and deep-lane implementation modules.
- Convert selected task deep wrappers to dispatcher symlinks.
- Add deterministic non-Kolme dispatcher fallback reason taxonomy markers.

Out of scope:
- Additional wrapper families outside selected task deep-lane tranche.

## Acceptance Criteria

AC-1:
Given task deep-lane wrappers, when invoked, then wrappers dispatch to deep-lane manifests through
`run_non_kolme_contract_lane_dispatch.sh`.

AC-2:
Given dispatcher unknown-wrapper failures, when observed in stderr, then deterministic fallback
status/taxonomy/reason markers are emitted.

AC-3:
Given migrated deep lanes, when executed, then pre-existing deep-lane functional behavior remains
unchanged.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Tests:
    - `bash scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
    - `bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`

- C-02 (AC-2, Regression):
  - Test:
    - `bash scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`

- C-03 (AC-3, Integration):
  - Tests:
    - `bash scripts/task/run_task_operation_snapshot_deep_lane.sh`
    - `bash scripts/task/run_federated_delegation_settlement_deep_lane.sh --output-json /tmp/federated-delegation-settlement-report.json`

## Success Metrics / Observable Signals

- Task deep wrapper family migrates to dispatcher path with deterministic fallback diagnostics.
