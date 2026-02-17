# Plan — #4301

Status: Reviewed

## Approach

- Implement envelope marker extensions and bound checks in
  `local_retry_diagnostics_live_contract.py`.
- Wire deterministic reason-code projection for reconnect/backoff/exhaustion drift.
- Update contract-lane wrappers/tests and docs/test contracts.

## Affected Areas

- `scripts/runtime/local_retry_diagnostics_live_contract.py`
- `scripts/runtime/validate_local_retry_diagnostics_live_contract_lane.sh`
- `scripts/runtime/test_validate_local_retry_diagnostics_live.sh`
- `scripts/runtime/test_check_local_retry_diagnostics_live_policy.sh`
- `scripts/runtime/test_validate_local_retry_diagnostics_live_contract_lane.sh`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations

- Risk: compatibility drift with existing local retry lane markers.
  - Mitigation: additive marker evolution and synchronized test updates.
