# Plan: Issue #4345

Status: Completed
Issue: #4345

## Approach

1. Extend task wrapper parity tests with deep-wrapper symlink + manifest assertions.
2. Extend framework wrapper matrix unknown-wrapper path to assert deterministic fallback markers.
3. Capture RED failures before implementing migration.

## Affected Modules

- `scripts/task/test_run_task_operation_snapshot_contract_lane.sh`
- `scripts/task/test_run_federated_delegation_settlement_contract_lane.sh`
- `scripts/framework/test_non_kolme_wave19_lightweight_contract_lane_dispatch_wrapper_matrix.sh`

## Risks / Mitigations

- Risk: RED checks accidentally pass due to existing behavior.
  - Mitigation: assert exact marker values not currently emitted.

## Interfaces / Contracts

- Expected fallback marker constants:
  - `fallback_reason_taxonomy_version=kamn.framework.non-kolme-dispatch-fallback-reason-taxonomy.v1`
  - `fallback_reason_codes_csv=dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped`
  - `fallback_reason_code=dispatcher_unknown_wrapper`

## ADR

No ADR required.
