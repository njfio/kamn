# Tasks: Issue #4346

Status: Completed
Issue: #4346

## Ordered Tasks

T1 (GREEN):
- Add deep manifests + impl scripts for task wrapper family.

T2 (GREEN):
- Convert deep wrappers to dispatcher symlinks.

T3 (GREEN):
- Extend non-Kolme dispatcher mapping, phase resolution, and deterministic fallback markers.

T4 (Verify):
- Run:
  - `bash scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
  - `bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
  - `bash scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`

## TDD Evidence

- RED input:
  - RED contracts introduced in #4345 were already failing deterministically prior to implementation.

- GREEN command/output:
  - `bash scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
    - Passed: `task operation snapshot contract lane script tests passed.`
  - `bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
    - Passed: `federated delegation settlement contract lane wrapper tests passed.`
  - `bash scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
    - Passed: `non-Kolme wave-19 lightweight contract lane dispatcher wrapper matrix tests passed.`
  - `bash scripts/task/run_task_operation_snapshot_deep_lane.sh`
    - Passed: `task operation snapshot deep lane tests passed.`
  - `bash scripts/task/test_run_federated_delegation_settlement_deep_lane.sh`
    - Passed: `federated delegation settlement deep lane script tests passed.`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed

- Regression summary:
  - Deep wrappers now route through the shared non-Kolme dispatcher with deep manifests.
  - Unknown-wrapper fallback now emits deterministic taxonomy/reason markers.
