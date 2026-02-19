# Plan — #4378

## Approach
- Define a bounded provider-failure taxonomy constant set in checker.
- Normalize observed provider-failure reason values by taxonomy order.
- Emit taxonomy fields in JSON report and stdout key-value output.
- Update tests/docs to assert new taxonomy markers.

## Risks
- Risk: taxonomy overlap with non-provider reasons.
  - Mitigation: classify provider-failure reasons via explicit constant set intersection.

## Interfaces
- `provider_failure_reason_taxonomy_version`
- `provider_failure_reason_codes_csv`
- `provider_failure_reason_codes_value`

## ADR
- Not required.
