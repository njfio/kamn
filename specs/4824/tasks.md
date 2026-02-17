# Tasks — Issue #4824

- [x] T1 (Red): add failing topology contract test before implementation.
  - Added `scripts/ci/test_non_kolme_wave_budget_trend_runner_contract.sh`.
  - RED evidence:
    - `bash scripts/ci/test_non_kolme_wave_budget_trend_runner_contract.sh`
    - Failure marker: `expected shared non-Kolme wave budget trend checker runner`

- [x] T2 (Green): implement shared parameterized checker runner + wave symlink entrypoints.
  - Added `scripts/ci/check_non_kolme_wave_wrapper_family_budget_trend_impl.sh`.
  - Converted wave1-wave19 checker scripts to symlinks targeting shared runner.

- [x] T3 (Refactor): preserve special-case behavior and deterministic contracts.
  - Preserved wave-19 max-runtime override handling.
  - Preserved checker command interface and deterministic output contract compatibility.

- [x] T4 (Verify): run conformance + integration suites and capture evidence.
  - `bash scripts/ci/test_non_kolme_wave_budget_trend_runner_contract.sh`
  - `for wave in {1..19}; do bash scripts/ci/test_check_non_kolme_wave${wave}_wrapper_family_budget_trend.sh; done`
  - `bash scripts/ci/test_ci_tools.sh`
