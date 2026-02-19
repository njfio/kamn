# Plan: Issue #4346

Status: Completed
Issue: #4346

## Approach

1. Add deep-lane manifests for task snapshot and federated delegation settlement wrappers.
2. Move current deep-lane shell logic into `*_impl.sh` modules.
3. Convert wrapper entrypoints to dispatcher symlinks.
4. Extend non-Kolme dispatcher with:
   - wrapper-to-manifest mapping for new deep wrappers,
   - deep phase mapping,
   - deterministic fallback marker emission.
5. Validate through wrapper parity and deep-lane behavioral tests.

## Affected Modules

- `scripts/framework/run_non_kolme_contract_lane_dispatch.sh`
- `scripts/framework/manifests/task_task_operation_snapshot_deep_lane.json`
- `scripts/framework/manifests/task_federated_delegation_settlement_deep_lane.json`
- `scripts/task/run_task_operation_snapshot_deep_lane.sh`
- `scripts/task/run_task_operation_snapshot_deep_lane_impl.sh`
- `scripts/task/run_federated_delegation_settlement_deep_lane.sh`
- `scripts/task/run_federated_delegation_settlement_deep_lane_impl.sh`

## Risks / Mitigations

- Risk: Symlink wrapper compatibility regressions.
  - Mitigation: keep wrapper filenames unchanged and assert `--resolve-manifest-path` contracts.
- Risk: Additional stderr markers impact callers.
  - Mitigation: markers are additive and aligned to existing fallback-pattern contracts.

## Interfaces / Contracts

- Fallback taxonomy version:
  - `kamn.framework.non-kolme-dispatch-fallback-reason-taxonomy.v1`

## ADR

No ADR required.
