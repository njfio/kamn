# Plan — #4242 Red Tests for Replay Taxonomy and Runbook Divergence

Status: Reviewed

## Approach

1. Extend `test_check_sqlite_crash_recovery_live_policy.sh` with replay taxonomy and runbook
   divergence checks that are expected to fail prior to implementation.
2. Extend `test_validate_sqlite_crash_recovery_live_contract_lane.sh` marker assertions for replay
   taxonomy/runbook parity outputs.
3. Keep assertions focused on deterministic reason markers and marker keys.

## Affected Surfaces

- `scripts/runtime/test_check_sqlite_crash_recovery_live_policy.sh`
- `scripts/runtime/test_validate_sqlite_crash_recovery_live_contract_lane.sh`

## Risks and Mitigations

- Risk: shell red tests become brittle due unrelated marker ordering.
  Mitigation: assert line markers and reason code containment, not full output ordering.
