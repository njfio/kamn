# Issue #3654 Tasks

- Issue: `#3654`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing policy taxonomy and lane marker checks.
- T2 (Green): keep policy extraction boundaries and deterministic reason codes.
- T3 (Regression): run policy/emulator/fallback marker suites.
- T4 (Verify): run
  - `cargo test -p kamn-node --test signer_policy_reason_taxonomy_contract`
  - `bash scripts/signer/test_run_signer_policy_contract_lane.sh`
  - `bash scripts/signer/test_run_signer_emulator_contract_lane.sh`
  - `bash scripts/kolme/test_check_fallback_signer_marker_matrix_policy.sh`

## Completion Evidence
- Signer policy module behavior and deterministic taxonomy checks are green.
