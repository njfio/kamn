# Plan — #4289

Status: Reviewed

## Approach

- Extend failover preflight contract-lane tests with convergence-specific RED fixtures:
  - missing evidence-link marker fixture
  - tampered promotion reason mapping fixture
  - repeated mismatch-order fixture

## Affected Areas

- `scripts/runtime/test_run_failover_sync_drill_preflight_contract_lane.sh`

## Risks and Mitigations

- Risk: false-positive test failures from unstable output.
  - Mitigation: assert deterministic reason ordering across repeated checks.
