# Plan — #4229

Status: Reviewed

## Approach

- Add dedicated test script for service-api-axum evidence convergence checker.
- Add missing-link, payload-tamper, and reason-mapping tamper fixtures.
- Ensure deterministic fail-closed reason assertions.

## Affected Areas

- `scripts/runtime/test_check_service_api_axum_ingress_live_evidence_convergence.sh`
- `scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`

## Risks and Mitigations

- Risk: test flakiness due non-deterministic reason order.
  - Mitigation: assert deterministic order and run repeated fixture checks for stability.
