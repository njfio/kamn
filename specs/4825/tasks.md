# Tasks — Issue #4825

- [x] T1 (Red): Add failing migration contract test.
  - Added `scripts/lib/test_test_harness_migration_contract.sh`.
  - RED evidence: failed before harness existed with missing `scripts/lib/test_harness.sh`.
- [x] T2 (Green): Implement shared harness + family-shared impl scripts.
  - Added `scripts/lib/test_harness.sh`.
  - Added `scripts/ci/test_wave_wrapper_family_baseline_contract_impl.sh`.
  - Added `scripts/ci/test_wave_wrapper_family_budget_trend_impl.sh`.
  - Converted family impl entrypoints to thin `exec` wrappers.
- [x] T3 (Refactor): Migrate representative framework script checks to harness helpers.
  - Updated `scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh`.
- [x] T4 (Verify): Refresh baseline + execute conformance and regression suites.
  - Refreshed `fixtures/ci/non_kolme_wave_trend_test_loc_soft_budget_baseline.json` (`total_shell_loc: 27`).
  - Conformance checks:
    - `bash scripts/lib/test_test_harness_migration_contract.sh`
    - Migrated wrapper cohort loops (non-Kolme waves 1-19, Kolme waves 8/10/11, framework waves 10-19)
    - `bash scripts/ci/test_check_non_kolme_wave_trend_test_loc_soft_budget.sh`
    - `bash scripts/ci/test_ci_tools.sh`
