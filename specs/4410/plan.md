# Plan — #4410

Status: Reviewed

## Approach

- Refactor runtime observability policy checker to avoid early generic failure on missing fields; emit deterministic reason-code mapping through policy decision accumulator.
- Add normalized `reason_codes_value` marker to policy JSON and CLI output for deterministic machine parsing.
- Update policy and contract-lane tests for new marker and missing-field reason mapping.
- Update `docs/observability/schema.md` with runtime observability endpoint payload matrix and taxonomy markers.

## Affected Areas

- `scripts/runtime/runtime_observability_endpoint_live_contract.py`
- `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
- `docs/observability/schema.md`

## Risks and Mitigations

- Risk: missing-field handling may produce additional reasons when multiple invariants are violated.
  - Mitigation: require deterministic presence of specific missing-field reason marker in tests.

