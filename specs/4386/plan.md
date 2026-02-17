# Plan — #4386

Status: Reviewed

## Approach

- Add RED coverage for lifecycle limit breach and unstable backpressure marker outcomes.
- Extend service API axum ingress policy checker with deterministic lifecycle taxonomy validation and normalized reason output (`reason_codes_value`).
- Promote lifecycle taxonomy markers into validation summary and contract-lane marker sets.
- Update service API and ops docs to keep lifecycle/backpressure contracts and validation commands aligned.

## Affected Areas

- `scripts/runtime/service_api_axum_ingress_live_contract.py`
- `scripts/runtime/validate_service_api_axum_ingress_live.sh`
- `scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh`
- `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
- `scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
- `docs/service/api-contract.md`
- `docs/ops/configuration.md`

## Risks and Mitigations

- Risk: tightening required markers could fail existing lanes due to missing fields.
  - Mitigation: introduce deterministic required-field reason mapping and update tests/lane expectations in the same change.
- Risk: marker drift between validation and policy outputs.
  - Mitigation: assert both shell output markers and JSON payload parity in policy and contract-lane tests.

## Interfaces / Contracts

- Summary marker additions:
  - `async_lifecycle_backpressure_projection_status`
  - `service_api_lifecycle_rejection_reason_taxonomy_version`
  - `service_api_lifecycle_rejection_reason_codes_csv`
- Policy marker additions:
  - `reason_codes_value`
  - `service_api_lifecycle_rejection_reason_taxonomy_version`
  - `service_api_lifecycle_rejection_reason_codes_csv`
