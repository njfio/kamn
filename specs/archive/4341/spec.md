# Spec: Issue #4341

Status: Implemented
Issue: #4341
Parent: #4339
Milestone: R27.32 Script-surface consolidation, documentation graduation, and architecture-navigability governance
Priority: P1

## Problem Statement

Task deep-lane wrappers are still bespoke scripts instead of manifest-dispatched wrappers, and the
shared non-Kolme dispatcher lacks deterministic fallback reason taxonomy markers for unknown
wrappers.

## Scope

In scope:
- Migrate selected task deep-lane wrappers to dispatcher symlink + manifest-backed deep execution.
- Add deterministic fallback reason marker outputs on unknown wrapper dispatch failures.
- Add parity checks for migrated wrappers and fallback markers.

Out of scope:
- Script budget threshold governance (tracked in #4342).

## Acceptance Criteria

AC-1:
Given task deep-lane wrappers, when wrapper parity checks run, then wrappers are symlinks to the
shared dispatcher and resolve deep-lane manifests.

AC-2:
Given migrated deep-lane manifests, when dispatched, then deep-lane implementation modules execute
successfully with no behavior regression.

AC-3:
Given unknown wrapper dispatches, when dispatcher rejects them, then deterministic fallback reason
taxonomy markers are emitted.

## Conformance Cases

- C-01 (AC-1, Functional):
  - Test: `bash scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
  - Expectation: deep wrapper symlink and manifest/impl mapping are asserted.

- C-02 (AC-1, Functional):
  - Test: `bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
  - Expectation: deep wrapper symlink and manifest/impl mapping are asserted.

- C-03 (AC-2, Integration):
  - Tests:
    - `bash scripts/task/run_task_operation_snapshot_deep_lane.sh`
    - `bash scripts/task/run_federated_delegation_settlement_deep_lane.sh --output-json /tmp/federated-delegation-settlement-report.json`
  - Expectation: deep-lane runtime behavior still passes.

- C-04 (AC-3, Regression):
  - Test: `bash scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
  - Expectation: unknown wrapper dispatch failure emits deterministic fallback markers.

## Success Metrics / Observable Signals

- Selected task deep-lane wrapper family is manifest-dispatched.
- Unknown-wrapper fallback diagnostics are deterministic and checker-validated.
