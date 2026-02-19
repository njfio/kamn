# Plan — #4394

Status: Implemented

## Approach

- Extend `test_validate_live_transport_fault_matrix_live.sh` with peer reason marker assertions.
- Extend `test_check_live_transport_fault_matrix_live_policy.sh` with deterministic peer reason marker and tamper-failure assertions.
- Extend `test_validate_live_transport_fault_matrix_live_contract_lane.sh` for docs-parity marker assertions.
- Capture RED failure output before implementation wiring.

## Affected Areas

- `scripts/runtime/test_validate_live_transport_fault_matrix_live.sh`
- `scripts/runtime/test_check_live_transport_fault_matrix_live_policy.sh`
- `scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh`

## Risks and Mitigations

- Risk: tests may only catch generic failure text.
  - Mitigation: assert explicit deterministic reason-code markers.
