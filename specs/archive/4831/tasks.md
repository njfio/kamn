# Tasks — Issue #4831

- [x] T1 (Red): add failing telemetry collector contract test before implementation.
  Evidence:
  - `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh` failed with:
    `expected shell-rust LOC telemetry collector to be executable: .../scripts/ci/collect_shell_rust_loc_telemetry.sh`
- [x] T2 (Green): implement fail-closed shell-rust LOC telemetry collector with deterministic taxonomy markers.
  Evidence:
  - Added `scripts/ci/collect_shell_rust_loc_telemetry.sh`.
  - Added deterministic report schema `kamn.ci.shell-rust-loc-telemetry-report.v1`.
  - Added deterministic taxonomy `kamn.ci.shell-rust-loc-telemetry-reason-taxonomy.v1`.
- [x] T3 (Refactor): wire collector coverage into CI tools regression and document contract.
  Evidence:
  - Added `scripts/ci/test_collect_shell_rust_loc_telemetry.sh`.
  - `scripts/ci/test_ci_tools.sh` now runs collector test in fast/full blocks.
  - `docs/ci/strategy.md` updated with collector command and reason-marker contract.
- [x] T4 (Verify): run deterministic suites and regression coverage.
  Evidence:
  - `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh`
  - `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh`
  - `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
