# Tasks: Issue #4342

Status: Completed
Issue: #4342

## Ordered Tasks

T1 (RED, Conformance):
- Extend trend checker tests to require taxonomy + runtime budget markers and over-budget fail-closed path.

T2 (GREEN, Implementation):
- Add taxonomy markers + runtime budget enforcement to wrapper trend checker and wave-19 wrapper.

T3 (GREEN, Docs):
- Update `docs/ci/strategy.md` taxonomy/budget policy marker references.

T4 (Verify):
- Run:
  - `bash scripts/ci/test_check_non_kolme_wave19_wrapper_family_budget_trend.sh`
  - `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/ci/check_non_kolme_wave19_wrapper_family_budget_trend.sh --matrix-file fixtures/ci/non_kolme_wave19_wrapper_family_matrix.json --baseline-file fixtures/ci/non_kolme_wave19_wrapper_family_baseline.json`
    - Output lacked taxonomy/budget markers (`reason_taxonomy_version`, `reason_codes_value`, `policy_decision`, `ci_smoke_budget_status`).
  - `bash scripts/ci/check_non_kolme_wave19_wrapper_family_budget_trend.sh --matrix-file fixtures/ci/non_kolme_wave19_wrapper_family_matrix.json --baseline-file fixtures/ci/non_kolme_wave19_wrapper_family_baseline.json --max-runtime-seconds 0`
    - Failed with parser rejection before implementation: `unrecognized arguments: --max-runtime-seconds 0`.

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
  - Trend checker now emits deterministic taxonomy and runtime-budget markers.
  - CI smoke runtime budget overruns fail closed with `ci_smoke_runtime_budget_exceeded`.
