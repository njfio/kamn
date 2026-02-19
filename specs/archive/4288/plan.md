# Plan — #4288

Status: Reviewed

## Approach

- Extend `check-policy` mode in failover preflight shared contract module with:
  - taxonomy parity marker validation
  - runbook marker parity validation against runbook file content
  - deterministic reason mapping and output serialization
- Keep default lane execution behavior unchanged.

## Affected Areas

- `scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh`
