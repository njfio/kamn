# Tasks - Issue #3966

- [x] T1 (Red): identify missing parent task lifecycle artifacts and parity gaps from child migrations.
- [x] T2 (Green): complete first-family migration and legacy-entrypoint parity contracts (via `#3970`, `#3971`).
- [x] T3 (Refactor/Docs): align strategy docs and command-surface contracts to shared dispatcher governance.
- [x] T4 (Verify): run focused wrapper-family, compatibility, strategy, command-surface, and fast-gate integration checks.

## Planned Verification Commands

- `bash scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh`
- `bash scripts/ci/test_non_kolme_wave1_wrapper_family_baseline_contract.sh`
- `bash scripts/ci/test_wrapper_dispatch_legacy_entrypoint_compatibility.sh`
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
