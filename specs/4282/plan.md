# Plan — #4282

Status: Reviewed

## Approach

- Extend failover preflight shared contract module with taxonomy/runbook parity marker outputs and policy checks.
- Add RED tests first in `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh` for:
  - taxonomy marker drift rejection
  - runbook marker divergence rejection
  - deterministic repeated-output behavior
- Implement runbook parity validation using a configurable runbook file path in policy-check mode.
- Update docs and docs-contract tests:
  - `docs/foundation/release-gonogo-checklist.md`
  - `docs/deploy/kolme_devnet_ops.md`
  - `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
  - `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Affected Areas

- `scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh`
- `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/deploy/kolme_devnet_ops.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Risks and Mitigations

- Risk: checker/doc marker mismatch introduced while extending taxonomy set.
  - Mitigation: enforce exact marker assertions in docs-contract tests.
- Risk: policy-check mode complexity increases.
  - Mitigation: keep deterministic marker constants centralized in the shared contract module.

## Interfaces and Contracts

- Added policy-check input:
  - `--runbook-file <path>` for parity validation against runbook marker declarations.
- Taxonomy parity marker set:
  - deterministic version/csv markers
  - deterministic status markers for taxonomy and runbook parity
