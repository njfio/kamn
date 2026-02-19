# Plan — #4392

Status: Implemented

## Approach

- Extend `test_check_service_api_websocket_live_policy.sh` with failing assertions for normalized reason output and missing-field/taxonomy drift.
- Extend `test_validate_service_api_websocket_live_contract_lane.sh` to assert policy `reason_codes_value` parity in integration flow.
- Capture failing RED outputs before implementation changes.

## Affected Areas

- `scripts/runtime/test_check_service_api_websocket_live_policy.sh`
- `scripts/runtime/test_validate_service_api_websocket_live_contract_lane.sh`

## Risks and Mitigations

- Risk: tests assert generic failures rather than deterministic reason codes.
  - Mitigation: match explicit reason-code markers in CLI output.
