# Tasks — Issue #4863

- [x] T1 (Red): child-task conformance tests expanded to fail on governance-marker drift and ratchet/telemetry reason-code regressions.
- [x] T2 (Green): deterministic ratchet policy + telemetry/PR declaration governance behavior implemented and validated.
- [x] T3 (Refactor): parent story rolled up with stable spec/task evidence paths to keep governance checks maintainable.
- [x] T4 (Verify): child task closures merged and story-level conformance evidence consolidated.

## Verification Evidence

- Task rollup PRs: `#4898` (task #4870), `#4899` (task #4871).
- Implementation PRs in story scope: `#4896`, `#4897`, `#4894`, `#4895`.
- Representative verification commands recorded in child-task specs:
  - `bash scripts/ci/test_check_fast_gate_budget_delta_threshold.sh`
  - `bash scripts/ci/test_run_fast_gate_budget_delta_contract_lane.sh`
  - `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh`
  - `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
  - `bash scripts/ci/test_check_pr_ci_declaration.sh`
  - `bash scripts/ci/test_pr_template_shell_surface_markers_contract.sh`
  - `bash scripts/ci/test_shell_surface_issue_intake_contract.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh`
