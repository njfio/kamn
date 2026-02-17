# Plan — #4286

Status: Reviewed

## Approach

- Implement `check-policy` mode in `scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh`.
- Validate required fields, marker statuses, expected decision, CI fast gate, and failover readiness taxonomy markers.
- Emit deterministic stdout markers and JSON policy output on both pass/fail paths.

## Affected Areas

- `scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh`

## Risks and Mitigations

- Risk: backwards compatibility for lane execution path.
  - Mitigation: keep legacy run behavior as default mode and branch early for policy mode.
