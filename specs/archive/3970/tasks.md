# Tasks - Issue #3970

- [x] T1 (Red): require wave-1 canary wrapper coverage in non-Kolme dispatcher wrapper-matrix contract.
- [x] T2 (Green): update wrapper-matrix test to validate canary + governance symlink dispatch parity and manifest resolution.
- [x] T3 (Docs): update CI strategy migration/parity documentation for wave-1 canary coverage.
- [x] T4 (Verify): run focused wave-1 baseline, wrapper matrix, strategy contract, and fast-gate CI tool checks.

## Planned Verification Commands

- `bash scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh`
- `bash scripts/ci/test_non_kolme_wave1_wrapper_family_baseline_contract.sh`
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
