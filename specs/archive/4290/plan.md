# Plan — #4290

Status: Reviewed

## Approach

- Add convergence-check mode to the failover preflight shared contract script.
- Extend policy output with deterministic promotion reason mapping markers.
- Emit convergence checker report markers for success/failure classification.
- Keep reason ordering deterministic by fixed insertion order and normalized payload checks.

## Affected Areas

- `scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh`
- `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh`

## Risks and Mitigations

- Risk: accidental nondeterminism in reason ordering.
  - Mitigation: append-only, ordered reason checks and repeated-run test assertions.
- Risk: marker drift across docs.
  - Mitigation: docs-contract tests updated in same PR.
