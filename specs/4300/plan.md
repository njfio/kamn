# Plan — #4300

Status: Reviewed

## Approach

- Expand existing runtime retry diagnostics shell tests to include tamper cases for:
  - retry-envelope exhaustion fail-closed marker,
  - reconnect attempt bound marker,
  - backoff window bound marker,
  - reason taxonomy/version and reason-csv markers.
- Execute these tests before implementation changes to establish RED evidence.

## Affected Areas

- `scripts/runtime/test_validate_local_retry_diagnostics_live.sh`
- `scripts/runtime/test_check_local_retry_diagnostics_live_policy.sh`
- `scripts/runtime/test_validate_local_retry_diagnostics_live_contract_lane.sh`

## Risks and Mitigations

- Risk: brittle string assertions.
  - Mitigation: assert deterministic marker keys/values and reason codes only.
