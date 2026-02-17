# Plan — #4409

Status: Reviewed

## Approach

- Extend `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh` with RED fixtures for missing required fields.
- Add assertions for normalized reason output marker(s) on GO/NO-GO paths.
- Keep red fixtures narrow so failures isolate policy-checker gaps.

## Affected Areas

- `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`

## Risks and Mitigations

- Risk: added red fixtures may fail for unrelated reasons.
  - Mitigation: mutate one field at a time from known-good report fixture.

