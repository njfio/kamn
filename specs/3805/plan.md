# Issue #3805 Plan

- Issue: `#3805`
- Status: `Completed`

## Approach
- Extend observability endpoint integration tests with TLS negative matrix selectors for missing cert, invalid key, invalid mode, and plain HTTP handshake rejection under TLS mode.
- Extend runtime validation report schema with deterministic TLS negative matrix marker.
- Extend policy and contract lane checks/tamper drills to enforce TLS negative matrix marker fail-closed behavior.
- Update runtime network docs with explicit observability TLS negative-path taxonomy.

## Affected Modules
- `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs`
- `scripts/runtime/validate_runtime_observability_endpoint_live.sh`
- `scripts/runtime/runtime_observability_endpoint_live_contract.py`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live.sh`
- `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
- `scripts/runtime/validate_runtime_observability_endpoint_live_contract_lane.sh`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
- `docs/foundation/runtime-network.md`

## Risks and Mitigations
- Risk: TLS negative tests introduce flaky socket/handshake timing.
- Mitigation: keep deterministic local loopback setup, bounded waits, and strict request budgets.
- Risk: marker proliferation drifts lane contracts.
- Mitigation: update required fields in policy checker and add tamper tests for deterministic rejection reasons.

## Interface Contract
- No CLI contract changes.
- Existing observability TLS env interface remains:
  - `KAMN_OBSERVABILITY_ENDPOINT_TLS_MODE=disabled|require`
  - `KAMN_OBSERVABILITY_ENDPOINT_TLS_CERT_FILE`
  - `KAMN_OBSERVABILITY_ENDPOINT_TLS_KEY_FILE`
- Runtime lane output adds deterministic marker:
  - `observability_tls_negative_matrix_status=verified`

## ADR
- No ADR required (no new dependency/protocol shape change).
