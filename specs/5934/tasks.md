# Tasks: Issue #5934 - Task: Reduce shell/python surface below policy ceiling and improve governance ratio

- Issue: #5934
- Spec: `specs/5934/spec.md`
- Plan: `specs/5934/plan.md`
- Status: Completed
- Last Updated: 2026-02-25

## Ordered Tasks
- T1 (RED / Conformance): ✅ added failing governance telemetry assertions in:
  - `scripts/ci/test_generate_combined_shell_surface_trend_report.sh`
  - `scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
- T2 (GREEN / Implementation): ✅ implemented governance telemetry + policy enforcement in:
  - `scripts/ci/generate_combined_shell_surface_trend_report.sh`
  - `scripts/ci/check_combined_shell_surface_trend_policy.sh`
- T3 (Refactor): ✅ synchronized synthetic contract-lane payloads with expanded report schema:
  - `scripts/ci/ignored_test_and_script_budget_trend_contract_lane_impl.sh`
- T4 (Regression): ✅ refreshed and verified ignored-test fixtures:
  - `fixtures/ci/ignored_test_inventory_baseline.json`
  - `fixtures/ci/ignored_test_inventory_metadata.json`
- T5 (Verify): ✅ executed:
  - `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh`
  - `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
  - `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh`
  - `bash scripts/ci/test_check_ignored_test_inventory_drift.sh`
  - `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
- T6 (Process): ✅ updated lifecycle artifacts and milestone index for implemented status.
