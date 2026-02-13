# Missing-Docs Velocity Metrics and Cadence

This note defines deterministic velocity metrics for `kamn-core` docs graduation
and the reporting cadence for issue `#2127`.

## Metrics

- `commit_delta`: `report.commit_count - baseline.commit_count`
- `graduated_module_delta`:
  `report.graduated_module_count - baseline.graduated_module_count`
- `observed_window_modules_per_100_commits`:
  `(graduated_module_delta * 100) / commit_delta` (0 when `commit_delta = 0`)
- `stagnation_window_exceeded`:
  `commit_delta >= max_commits_without_graduation` and
  `graduated_module_delta == 0`
- `window_target_met`:
  `observed_window_modules_per_100_commits >= min_modules_per_100_commits`
- `allowlist_exhausted`: `report.allowlisted_module_count == 0`
  - terminal behavior: when true, policy returns
    `reason_key=allowlist_fully_graduated` with `final_decision=GO`
    and skips stagnation/velocity-window threshold checks.

## Reporting Cadence

- PR fast gate:
  - `bash scripts/ci/test_missing_docs_velocity_guard_contract.sh`
  - `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- Local evidence capture:
  - `python3 scripts/ci/missing_docs_throughput_report_contract.py generate --output-json /tmp/kamn-core-missing-docs-throughput-report.json`
  - `python3 scripts/ci/missing_docs_velocity_guard.py check --report-file /tmp/kamn-core-missing-docs-throughput-report.json --baseline-file fixtures/ci/kamn_core_missing_docs_velocity_baseline.json --threshold-file .ci/kamn-core-missing-docs-velocity-thresholds.json --output-json /tmp/kamn-core-missing-docs-velocity-policy.json`
- Weekly issue status rollup:
  - Post current `commit_delta`, `graduated_module_delta`, `reason_key`,
    and `final_decision` to the active docs-hardening story issue.

## Threshold Source of Truth

- Baseline:
  - `fixtures/ci/kamn_core_missing_docs_velocity_baseline.json`
- Thresholds:
  - `.ci/kamn-core-missing-docs-velocity-thresholds.json`
- Policy engine:
  - `scripts/ci/missing_docs_velocity_guard.py`

Regression: #2127
