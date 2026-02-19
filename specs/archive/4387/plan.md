# Plan — #4387

Status: Implemented

## Approach

- Extend websocket policy tests with RED assertions for normalized reason output and required-field drift handling.
- Update websocket policy checker to map missing required fields deterministically and emit `reason_codes_value`.
- Add docs parity validation in websocket live validation lane against release go/no-go checklist protocol/session markers.
- Update websocket contract-lane tests to assert policy normalized reason markers and docs parity markers.
- Update service API + release checklist docs where needed for explicit protocol/session marker references.

## Affected Areas

- `scripts/runtime/service_api_websocket_live_contract.py`
- `scripts/runtime/test_check_service_api_websocket_live_policy.sh`
- `scripts/runtime/validate_service_api_websocket_live.sh`
- `scripts/runtime/validate_service_api_websocket_live_contract_lane.sh`
- `scripts/runtime/test_validate_service_api_websocket_live.sh`
- `scripts/runtime/test_validate_service_api_websocket_live_contract_lane.sh`
- `docs/service/api-contract.md`
- `docs/foundation/release-gonogo-checklist.md`

## Risks and Mitigations

- Risk: stricter required-field checks may fail existing reports.
  - Mitigation: enforce deterministic required-field reason mapping and update tests/lane markers together.
- Risk: docs-parity checks introduce coupling to docs text.
  - Mitigation: verify explicit marker strings and keep docs updates in same PR.

## Interfaces / Contracts

- Policy output additions:
  - `reason_codes_value`
- Validation output additions:
  - `protocol_session_docs_contract_status`
  - `service_api_protocol_session_reason_taxonomy_version`
  - `service_api_protocol_session_reason_codes_csv`
