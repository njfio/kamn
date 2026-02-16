# Tasks: Issue #4345

Status: Completed
Issue: #4345

## Ordered Tasks

T1 (RED):
- Update tests to require migrated deep wrappers + deterministic fallback markers.

T2 (Capture RED Evidence):
- Run:
  - `bash scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
  - `bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
  - `bash scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`

## TDD Evidence

- RED command/output:
  - `bash scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
    - Failed with: `expected task operation snapshot deep-lane implementation module to be executable`
  - `bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
    - Failed with: `expected federated delegation settlement deep lane implementation module to be executable`
  - `bash scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
    - Failed with: `expected lightweight wrapper to be a symlink to shared dispatcher: .../scripts/task/run_task_operation_snapshot_deep_lane.sh`

- Regression summary:
  - RED checks now encode required deep-wrapper migration and fallback marker determinism contracts.
