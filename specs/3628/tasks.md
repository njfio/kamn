# Issue #3628 Tasks

- Issue: `#3628`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add signer boundary, extraction budget, taxonomy, and parity guard tests.
- T2 (Green): extract signer responsibilities into adapter/policy/parity-supporting module seams.
- T3 (Regression): run signer parity and startup lanes.
- T4 (Verify): run
  - `cargo test -p kamn-node --test signer_adapter_boundary_contract`
  - `cargo test -p kamn-node --test signer_extraction_budget_contract`
  - `cargo test -p kamn-node --test signer_policy_reason_taxonomy_contract`
  - `bash scripts/kolme/test_run_signature_parity_contract_lane.sh`
  - `bash scripts/kolme/test_run_nonce_broadcast_parity_contract_lane.sh`
  - `bash scripts/kolme/test_run_managed_signer_startup_live_validation_contract_lane.sh`

## Completion Evidence
- Signer decomposition and parity contracts pass across adapter/policy/startup/nonce surfaces.
