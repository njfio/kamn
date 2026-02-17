# Plan — #4296

Status: Reviewed

## Approach

- Extend `scripts/runtime/local_retry_diagnostics_live_contract.py` to model deterministic retry
  envelope and reconnect/backoff bounds with explicit markers.
- Add RED tests (subtask #4300) that fail on missing exhaustion fail-closed markers, reason taxonomy
  drift, and envelope bound drift.
- Implement policy-checker enforcement and reason projection (subtask #4301) for reconnect sequence
  bounds and retry envelope exhaustion markers.
- Update docs and docs-contract assertions for `docs/ops/configuration.md`.

## Affected Areas

- `scripts/runtime/local_retry_diagnostics_live_contract.py`
- `scripts/runtime/test_validate_local_retry_diagnostics_live.sh`
- `scripts/runtime/test_check_local_retry_diagnostics_live_policy.sh`
- `scripts/runtime/validate_local_retry_diagnostics_live_contract_lane.sh`
- `scripts/runtime/test_validate_local_retry_diagnostics_live_contract_lane.sh`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations

- Risk: changing reason taxonomy markers causes broad docs/CI drift.
  - Mitigation: deterministic constant updates and synchronized tests/docs.
- Risk: policy checker regressions on existing local retry lane behavior.
  - Mitigation: maintain existing markers while adding bounded-envelope markers and regression tests.

## Interfaces and Contracts

- Deterministic markers for reconnect envelope: attempt/backoff bounds, sequence status, exhaustion status.
- Policy reason taxonomy extension for retry envelope and reconnect bound drift.
