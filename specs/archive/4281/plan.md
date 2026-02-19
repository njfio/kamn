# Plan — #4281

Status: Reviewed

## Approach

- Extend `scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh` with a `check-policy` mode that validates preflight reports and emits deterministic fail-closed reason mapping.
- Add RED tests in `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh` for missing drift markers, drifted parity markers, and deterministic repeated-reason behavior.
- Keep implementation inside the existing shared contract module to avoid introducing extra shell wrapper sprawl.
- Update docs and docs-contract assertions:
  - `docs/ops/configuration.md`
  - `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Affected Areas

- `scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh`
- `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations

- Risk: introducing policy mode could break existing preflight-lane behavior.
  - Mitigation: keep default lane path unchanged; guard new behavior behind explicit `check-policy` mode.
- Risk: reason taxonomy drift across scripts/docs/tests.
  - Mitigation: use deterministic constants in script and assert exact strings in tests/docs.

## Interfaces and Contracts

- `check-policy` contract mode inputs:
  - `--report-file`, `--expected-final-decision`, `--ci-fast-gate`, `--output-json`
- Deterministic drift reason taxonomy anchor:
  - `kamn.runtime.failover-readiness-reason-taxonomy.v1`
- Stable reason taxonomy CSV:
  - `failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded`
