# Plan — #4393

Status: Implemented

## Approach

- Update websocket live policy checker to replace hard fail on missing fields with deterministic decision reasons.
- Add `reason_codes_value` to websocket policy output (JSON and CLI).
- Extend websocket validation lane to verify release checklist protocol/session markers and emit docs parity status.
- Wire new docs parity markers through websocket contract lane and tests.

## Affected Areas

- `scripts/runtime/service_api_websocket_live_contract.py`
- `scripts/runtime/validate_service_api_websocket_live.sh`
- `scripts/runtime/validate_service_api_websocket_live_contract_lane.sh`
- `docs/foundation/release-gonogo-checklist.md`

## Risks and Mitigations

- Risk: docs text drift could cause noisy failures.
  - Mitigation: validate only deterministic marker strings required by contracts.
- Risk: additional required markers could break existing lane behavior.
  - Mitigation: update lane and tests atomically with checker/validation changes.
