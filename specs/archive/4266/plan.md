# Plan — #4266

Status: Reviewed

## Approach

- Extend `service_api_axum_ingress_live_contract.py` with deterministic mismatch reason mapping outputs.
- Require mapping markers in the axum ingress contract lane wrapper.
- Add RED/REGRESSION tests for deterministic marker mismatch rejection and repeated-run ordering.
- Update ops/release docs and docs-contract assertions for mapping marker parity.

## Risks and Mitigations

- Risk: drift between policy output markers and docs.
  - Mitigation: add docs-contract assertions in Rust tests and lane shell tests.
- Risk: unstable reason ordering under multi-failure cases.
  - Mitigation: explicit repeated-run assertions using identical tampered fixtures.

## Interfaces and Contracts

- `scripts/runtime/service_api_axum_ingress_live_contract.py` (`check-policy`)
- Policy marker additions:
  - `service_api_axum_protocol_mismatch_reason_mapping_status`
  - `service_api_axum_protocol_mismatch_reason_taxonomy_version`
  - `service_api_axum_protocol_mismatch_reason_codes_csv`
  - `service_api_axum_protocol_mismatch_reason_code`
