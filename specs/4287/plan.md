# Plan — #4287

Status: Reviewed

## Approach

- Extend existing failover preflight test script with new red assertions for taxonomy and runbook parity drift.
- Use tampered report fixtures and tampered runbook fixture file to trigger deterministic fail-closed paths.
- Assert repeated output determinism via policy JSON reason-code comparison.

## Affected Areas

- `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh`
