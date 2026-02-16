# Plan: Issue #4339

Status: InProgress
Issue: #4339

## Approach

1. Create RED assertions for task deep-lane wrapper migration and fallback marker determinism.
2. Migrate task deep-lane wrappers to shared dispatcher with deep-lane manifests + impl modules.
3. Extend non-Kolme dispatcher to emit deterministic fallback reason taxonomy markers.
4. Update docs for wrapper migration status and fallback marker contract.
5. Verify targeted shell checks and supporting unit checks.

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
- `docs/ci/strategy.md`
- `docs/planning/next_steps_kolme_hardening.md`

## Risks / Mitigations

- Risk: Deep-lane phase mapping mismatch breaks wrapper execution.
  - Mitigation: add explicit dispatcher manifest and phase resolution tests.
- Risk: Unknown-wrapper fallback output changes could break existing callers.
  - Mitigation: additive stable markers with explicit regression checks.

## Interfaces / Contracts

- Fallback taxonomy markers:
  - `fallback_reason_taxonomy_version=kamn.framework.non-kolme-dispatch-fallback-reason-taxonomy.v1`
  - `fallback_reason_codes_csv=dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped`

## ADR

No ADR required.
