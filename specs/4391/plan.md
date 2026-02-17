# Plan — #4391

Status: Reviewed

## Approach

- Refactor service API axum ingress policy checker required-field handling into deterministic decision reasons.
- Add lifecycle taxonomy constants and validation checks to policy checker.
- Add normalized `reason_codes_value` output and include lifecycle taxonomy markers in policy output.
- Ensure validation script and contract lane emit/check corresponding lifecycle markers.

## Affected Areas

- `scripts/runtime/service_api_axum_ingress_live_contract.py`
- `scripts/runtime/validate_service_api_axum_ingress_live.sh`
- `scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh`

## Risks and Mitigations

- Risk: changing required-field handling could alter failure surface unexpectedly.
  - Mitigation: preserve existing reason-code semantics and add explicit deterministic required-field reason coverage.
- Risk: lifecycle taxonomy strings drift from docs.
  - Mitigation: hardcode deterministic constants and assert docs parity in tests/docs update.
