# Tasks: Issue #4339

Status: InProgress
Issue: #4339

## Ordered Tasks

T1 (RED, Conformance):
- Add failing assertions for dispatcher deep-lane migration + fallback marker determinism in:
  - `scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
  - `scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
  - `scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`

T2 (GREEN, Implementation):
- Migrate selected task deep wrappers to dispatcher + manifest/impl split.
- Add deterministic fallback reason taxonomy markers to non-Kolme dispatcher.

T3 (GREEN, Docs):
- Update migration + fallback marker references in:
  - `docs/ci/strategy.md`
  - `docs/planning/next_steps_kolme_hardening.md`

T4 (Verify):
- Run:
  - `bash scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
  - `bash scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
  - `bash scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`

