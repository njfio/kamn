# Issue #3658 Plan

- Issue: `#3658`
- Status: `Completed`

## Approach
- Add TLS serving support to observability endpoint runtime path using existing env-gated TLS pattern.
- Add an integration test that verifies `/metrics`, `/healthz`, `/readyz` over HTTPS.
- Extend runtime observability lane and local observability local-heavy lane with deterministic TLS route markers.
- Extend policy/contract tests to fail closed on TLS marker drift.
- Update CI strategy docs to reflect local-heavy TLS route coverage and markers.

## Affected Modules
- `crates/kamn-node/src/observability_endpoint.rs`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `scripts/runtime/validate_runtime_observability_endpoint_live.sh`
- `scripts/runtime/runtime_observability_endpoint_live_contract.py`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live.sh`
- `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
- `scripts/runtime/local_observability_scrape_live_contract.py`
- `scripts/runtime/test_validate_local_observability_scrape_live.sh`
- `scripts/runtime/test_check_local_observability_scrape_live_policy.sh`
- `scripts/runtime/validate_local_observability_scrape_live_contract_lane.sh`
- `scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh`
- `docs/ci/strategy.md`

## Risks and Mitigations
- Risk: TLS env wiring introduces regression in non-TLS observability mode.
- Mitigation: keep default mode disabled and preserve existing HTTP test selectors.
- Risk: local-heavy lane drift on command counts/markers.
- Mitigation: update policy/tests and keep deterministic marker validation.

## Interface Contract
- No CLI shape changes required.
- Observability TLS mode is env-gated and backward compatible:
  - `KAMN_OBSERVABILITY_ENDPOINT_TLS_MODE=disabled|require`
  - `KAMN_OBSERVABILITY_ENDPOINT_TLS_CERT_FILE`
  - `KAMN_OBSERVABILITY_ENDPOINT_TLS_KEY_FILE`

## ADR
- No ADR required.
