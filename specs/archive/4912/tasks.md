# Tasks — Issue #4912

- [x] T1 (Red): add failing conformance tests for shell hard-ceiling checker and spec-archive pointer policy.
- [x] T2 (Green): implement checker, CI wiring, spec archive move + pointer stubs, and legacy-wrapper deletion wave.
- [x] T3 (Refactor): centralize repeated shell LOC reporting logic and update docs/policy references.
- [x] T4 (Verify): run targeted and aggregate CI-tool checks and update issue lifecycle evidence.

## Verification Evidence

- `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`
- `bash scripts/ci/test_check_spec_archive_policy.sh`
- `bash scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh`
- `bash scripts/framework/test_non_kolme_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
