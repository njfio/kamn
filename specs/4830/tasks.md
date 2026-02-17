# Tasks — Issue #4830

- [x] T1 (Red): add failing drift contract test before checker implementation.
  Evidence:
  - `bash scripts/framework/test_check_lane_registry_drift.sh` failed with:
    `expected lane registry drift checker to be executable: .../scripts/framework/check_lane_registry_drift.sh`
- [x] T2 (Green): implement fail-closed lane registry drift checker.
  Evidence:
  - Added `scripts/framework/check_lane_registry_drift.sh` with deterministic reason taxonomy markers.
  - Added `scripts/framework/test_check_lane_registry_drift.sh` pass + tamper NO-GO coverage.
- [x] T3 (Refactor): wire drift checker contract into shared framework regression entrypoint and docs.
  Evidence:
  - `scripts/framework/test_contract_framework.sh` includes drift checker test.
  - `docs/architecture/lane-registry-generation.md` documents static maintenance retirement and drift checker usage.
- [x] T4 (Verify): run deterministic suites and regression coverage.
  Evidence:
  - `bash scripts/framework/test_check_lane_registry_drift.sh`
  - `bash scripts/framework/test_contract_framework.sh`
  - `bash scripts/framework/test_lane_registry_generation.sh`
  - `bash scripts/ci/test_ci_tools.sh`
