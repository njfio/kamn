# Issue #3636 Tasks

- Issue: `#3636`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add/retain failing adapter-boundary and extraction-ownership tests.
- T2 (Green): move crypto and key-source paths behind signer adapter boundaries.
- T3 (Regression): execute signature parity lane checks.
- T4 (Verify): run
  - `cargo test -p kamn-node --test signer_adapter_boundary_contract`
  - `cargo test -p kamn-node --test signer_extraction_budget_contract`
  - `bash scripts/kolme/test_run_signature_parity_contract_lane.sh`
  - `bash scripts/kolme/test_check_signature_parity_policy.sh`

## Completion Evidence
- Signer adapter boundary and parity checks pass after decomposition.
