# Tasks - Issue #3964

- [x] T1 (Red): identify missing story-level lifecycle artifacts and AC mapping gaps.
- [x] T2 (Green): complete child task delivery for wrapper migration parity (`#3966`) and shell governance (`#3967`).
- [x] T3 (Refactor/Docs): codify story-level AC->conformance mapping in repo specs.
- [x] T4 (Verify): run targeted wrapper parity, shell governance, and fast-gate integration checks.

## Planned Verification Commands

- `bash scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh`
- `bash scripts/ci/test_wrapper_dispatch_legacy_entrypoint_compatibility.sh`
- `bash scripts/ci/test_check_script_duplication_budget.sh`
- `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
- `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`
- `bash scripts/ci/test_run_fast_gate_budget_delta_contract_lane.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
