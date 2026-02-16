# Tasks: Issue #4341

Status: Completed
Issue: #4341

## Ordered Tasks

T1 (RED, Conformance):
- Add failing migration/fallback assertions in:
  - `scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
  - `scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
  - `scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`

T2 (GREEN, Implementation):
- Add task deep-lane manifests + implementation scripts.
- Convert deep wrappers to dispatcher symlinks.
- Extend dispatcher wrapper/phase mapping and deterministic fallback markers.

T3 (Verify):
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
  - Task deep wrappers now resolve deep manifests through shared non-Kolme dispatcher.
  - Unknown-wrapper fallback emits deterministic taxonomy/reason markers.
