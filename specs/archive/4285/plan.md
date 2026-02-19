# Plan — #4285

Status: Reviewed

## Approach

- Add tests directly in `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh` that invoke shared preflight contract module in policy-check mode.
- Introduce tampered report fixtures to assert deterministic fail-closed behavior for missing markers and drift mismatch.
- Validate deterministic repeated outputs by invoking policy check twice on the same tampered report and comparing reason outputs.

## Affected Areas

- `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh`

## Risks and Mitigations

- Risk: brittle string assertions.
  - Mitigation: assert stable reason markers and JSON reason arrays.
