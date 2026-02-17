# Plan — #4390

Status: Reviewed

## Approach

- Extend `test_check_service_api_axum_ingress_live_policy.sh` with lifecycle-limit and lifecycle-taxonomy tamper cases.
- Extend `test_validate_service_api_axum_ingress_live_contract_lane.sh` to assert lifecycle taxonomy markers and normalized reason output.
- Capture red-state expectations first, then keep assertions green after #4391 implementation.

## Affected Areas

- `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
- `scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`

## Risks and Mitigations

- Risk: tests accidentally assert generic errors instead of deterministic reason codes.
  - Mitigation: assert explicit reason code fragments and normalized marker lines.
- Risk: red tests only cover policy script and miss contract lane behavior.
  - Mitigation: add parity assertions in both policy-level and lane-level suites.
