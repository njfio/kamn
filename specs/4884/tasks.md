# Tasks — Issue #4884

- [x] T1 (Red): add conformance checks that fail when deterministic shell-rust telemetry/policy reason-code markers drift.
- [x] T2 (Green): implement test coverage for missing deterministic threshold-failure and warn-marker paths.
- [x] T3 (Refactor): keep fixtures/scenarios focused and re-use generated baseline reports for deterministic matrix coverage.
- [x] T4 (Verify): run targeted + fast-mode regression coverage and record evidence.

## Verification Evidence

- `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh` → `shell-rust LOC telemetry collector tests passed.`
- `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh` → `combined shell-surface trend policy checker tests passed.`
- `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh` → `Fast-mode CI tool regression tests passed.`
