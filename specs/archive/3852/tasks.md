# Tasks - Issue #3852

- [x] T1 (Red): define failing conformance checks for combined report generation and threshold enforcement (implemented in subtasks `#3853`/`#3854`).
- [x] T2 (Green): deliver combined report generator and threshold policy checker with deterministic markers.
- [x] T3 (Refactor/Docs): wire composed ignored-test/script contract lane and maintain harness-governance coverage.
- [x] T4 (Verify): run representative unit/functional/integration/regression checks across all governance surfaces.

## Planned Verification Commands

- `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh`
- `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
- `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh`
- `bash scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh`
