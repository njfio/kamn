# Issue #3653 Tasks

- Issue: `#3653`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing adapter-boundary and extraction-budget checks.
- T2 (Green): move key-source and crypto paths into signer adapter ownership.
- T3 (Regression): run signature parity matrix and policy checks.
- T4 (Verify): run
  - `cargo test -p kamn-node --test signer_adapter_boundary_contract`
  - `cargo test -p kamn-node --test signer_extraction_budget_contract`
  - `bash scripts/kolme/test_run_signature_parity_matrix.sh`
  - `bash scripts/kolme/test_check_signature_parity_policy.sh`

## Completion Evidence
- Signer adapter ownership and parity contracts are green.
