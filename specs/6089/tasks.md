# Tasks: Issue #6089

## Ordered Tasks
- T1 (Implementation): Add Python implementations for trend and baseline wave-wrapper harnesses.
- T2 (Implementation): Convert existing `.sh` impl scripts to thin compatibility wrappers that delegate to Python.
- T3 (Verification): Run representative wave harness checks:
  - `bash scripts/ci/test_check_kolme_wave10_wrapper_family_budget_trend.sh`
  - `bash scripts/ci/test_check_non_kolme_wave10_wrapper_family_budget_trend.sh`
  - `bash scripts/ci/test_kolme_wave10_wrapper_family_baseline_contract.sh`
  - `bash scripts/ci/test_non_kolme_wave10_wrapper_family_baseline_contract.sh`
- T4 (Regression): Run runner-contract checks:
  - `bash scripts/ci/test_kolme_wave_budget_trend_runner_contract.sh`
  - `bash scripts/ci/test_non_kolme_wave_budget_trend_runner_contract.sh`
- T5 (Closure): Record shell-surface delta markers in PR/issue closure.

## Tier Mapping
- Functional: T3
- Regression: T4
- Conformance: T3, T5
- Unit: N/A (script/harness migration)
- Integration: N/A (no runtime integration surface change)
