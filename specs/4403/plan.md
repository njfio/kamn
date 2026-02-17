# Plan — #4403

Status: Reviewed

## Approach

- Extend runtime observability endpoint policy tests with RED coverage for missing required fields and normalized reason output markers.
- Harden policy checker to map missing required fields into deterministic fail-closed reason codes (instead of generic parse failure text).
- Emit normalized reason marker value output (`reason_codes_value`) for policy payloads and CLI output.
- Update contract-lane assertions and observability schema docs to match new marker surface.

## Affected Areas

- `scripts/runtime/runtime_observability_endpoint_live_contract.py`
- `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
- `docs/observability/schema.md`

## Risks and Mitigations

- Risk: stricter missing-field handling may alter downstream error text expectations.
  - Mitigation: enforce stable deterministic reason code format and update tests/docs in same PR.
- Risk: marker drift between policy output and contract-lane expectations.
  - Mitigation: validate through both policy and contract-lane tests.

