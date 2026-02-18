# Tasks — Issue #4832

- [x] T1 (Red): add failing fast-gate workflow wiring contract test before workflow changes.
  Evidence:
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh` failed with:
    `expected combined shell-surface trend report generation step in ci-fast-gate workflow`
- [x] T2 (Green): wire ratio/policy/telemetry shell-surface checks into fast-gate workflow.
  Evidence:
  - Added generate/policy/telemetry steps in `.github/workflows/ci-fast-gate.yml` under `run_script_surface_budget_checks`.
  - Added combined trend + telemetry artifact upload steps.
- [x] T3 (Refactor): keep workflow/runtime bounded and reuse generated trend report.
  Evidence:
  - `collect_shell_rust_loc_telemetry.sh` supports `--report-file` to avoid duplicate report generation.
  - Fast-gate bounded runtime budget remains `timeout-minutes: 20`.
  - Added workflow wiring contract test to `scripts/ci/test_ci_tools.sh`.
- [x] T4 (Verify): run deterministic suites and regression coverage.
  Evidence:
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh`
  - `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh`
  - `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
