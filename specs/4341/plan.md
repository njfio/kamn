# Plan: Issue #4341

Status: Completed
Issue: #4341

## Approach

1. Add RED assertions to existing task and framework wrapper matrix tests.
2. Create deep-lane manifests and deep-lane implementation scripts for task wrapper family.
3. Convert task deep-lane wrapper scripts to dispatcher symlinks.
4. Extend dispatcher mapping/phase tables and deterministic fallback marker output.
5. Validate wrapper parity and deep-lane execution behavior.

## Affected Modules

- `scripts/framework/run_non_kolme_contract_lane_dispatch.sh`
- `scripts/framework/manifests/task_task_operation_snapshot_deep_lane.json`
- `scripts/framework/manifests/task_federated_delegation_settlement_deep_lane.json`
- `scripts/task/run_task_operation_snapshot_deep_lane.sh`
- `scripts/task/run_task_operation_snapshot_deep_lane_impl.sh`
- `scripts/task/run_federated_delegation_settlement_deep_lane.sh`
- `scripts/task/run_federated_delegation_settlement_deep_lane_impl.sh`
- `scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
- `scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
- `scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`

## Risks / Mitigations

- Risk: Dispatcher phase mapping regression for migrated wrappers.
  - Mitigation: assert `--resolve-manifest-path` and run migrated deep lanes in tests.
- Risk: Fallback markers drift.
  - Mitigation: assert exact marker values in wrapper matrix regression test.

## Interfaces / Contracts

- New manifest names:
  - `task_task_operation_snapshot_deep_lane.json`
  - `task_federated_delegation_settlement_deep_lane.json`
- Dispatcher fallback markers:
  - `dispatch_status=fail`
  - `fallback_reason_taxonomy_version=kamn.framework.non-kolme-dispatch-fallback-reason-taxonomy.v1`
  - `fallback_reason_codes_csv=dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped`
  - `fallback_reason_code=dispatcher_unknown_wrapper`

## ADR

No ADR required.
