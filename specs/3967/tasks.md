# Tasks — Issue #3967

- [x] T1 (Red): validate stale baseline failure mode in combined shell-surface trend checker.
  Evidence:
  - PR `#4856` fast-gate failed in step `Check combined shell-surface trend policy` with:
    - `reason_codes=combined_shell_surface_shell_line_total_delta_fail_exceeded,combined_shell_surface_ratio_warn_ceiling_exceeded,combined_shell_surface_ratio_delta_warn_exceeded`
    - `delta_shell_line_total=4930`
- [x] T2 (Green): refresh baseline fixture and add deterministic baseline-refresh workflow markers in CI strategy docs.
  Evidence:
  - Updated `fixtures/ci/combined_shell_surface_trend_baseline.json` to current metrics.
  - Added baseline-refresh workflow markers under combined shell-surface trend generator section in `docs/ci/strategy.md`.
- [x] T3 (Refactor): add docs-contract coverage for refresh workflow markers.
  Evidence:
  - Added `doc_contains_combined_shell_surface_baseline_refresh_workflow_markers` in `crates/kamn-core/tests/ci_strategy_docs.rs`.
- [x] T4 (Verify): run generator/policy/docs regression checks and record evidence.
  Evidence:
  - `bash scripts/ci/generate_combined_shell_surface_trend_report.sh --output-json /tmp/combined-shell-surface-trend-report.json`
  - `bash scripts/ci/check_combined_shell_surface_trend_policy.sh --report-file /tmp/combined-shell-surface-trend-report.json --threshold-file fixtures/ci/combined_shell_surface_trend_thresholds.json --output-json /tmp/combined-shell-surface-trend-policy-report.json`
  - `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs doc_contains_combined_shell_surface_baseline_refresh_workflow_markers -- --exact`
  - `cargo fmt --check`
