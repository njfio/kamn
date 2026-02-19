# Plan — #4302

Status: Reviewed

## Approach

- Extend existing unified local-heavy shell tests with new tamper drills for correlation schema and
  propagation parity.
- Start with assertions that are not yet implemented so the tests fail (RED baseline).
- Preserve existing deterministic marker assertions; append new checks instead of replacing old ones.

## Affected Areas

- `scripts/runtime/test_validate_unified_api_observability_local_heavy_live.sh`
- `scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`
- `scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh`

## Risks and Mitigations

- Risk: broad reason-code CSV updates break unrelated assertions.
  - Mitigation: update expected CSV constants in one pass with deterministic ordering.
- Risk: false negatives from test fixture drift.
  - Mitigation: tamper only one marker per drill and assert a single deterministic reason code.
