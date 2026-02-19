# Tasks: Issue #4348

Status: Completed
Issue: #4348

## Ordered Tasks

T1 (GREEN):
- Implement taxonomy/runtime marker emission and runtime budget fail-closed checks.

T2 (GREEN):
- Thread wave-19 wrapper runtime budget defaults and ensure deterministic outputs.

T3 (Verify):
- Run:
  - `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`
  - `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

## TDD Evidence

- RED input:
  - RED assertions from #4347 were failing before implementation (taxonomy markers missing; runtime-budget contract path absent).

- GREEN command/output:
  - `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`
    - Passed: `non-Kolme wave-19 wrapper-family budget trend checker tests passed.`
  - `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh`
    - Passed: `Ignored-test and script soft-budget trend contract lane tests passed.`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
    - Passed: `CI strategy contract tests passed.`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed

- Regression summary:
  - Trend checker now publishes deterministic taxonomy markers and runtime budget status markers.
  - Wave-19 trend wrapper now enforces explicit CI smoke runtime budget defaults.
