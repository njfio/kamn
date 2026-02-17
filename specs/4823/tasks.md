# Tasks — Issue #4823

- [x] T1 (Red): add failing topology contract test before implementation.
  - Added `scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh`.
  - RED evidence:
    - `bash scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh`
    - Failure marker: `expected shared non-Kolme wave lightweight wrapper matrix runner`

- [x] T2 (Green): implement shared runner + wave definition files + symlink entrypoints.
  - Added `scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh`.
  - Added wave definition files under `scripts/framework/wave_definitions/`.
  - Replaced wave10-wave19 scripts with symlinks to the shared runner.

- [x] T3 (Refactor): enforce deterministic unknown-wrapper fallback checks in shared path.
  - Shared runner validates deterministic fallback markers for all waves.
  - Removed duplicated per-wave script logic while preserving command compatibility.

- [x] T4 (Verify): run conformance and integration suites.
  - `bash scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh`
  - `for wave in {10..19}; do bash scripts/framework/test_non_kolme_wave${wave}_lightweight_contract_lane_dispatch_wrapper_matrix.sh; done`
  - `bash scripts/kolme/test_run_continuous_runtime_commit_contract_lane.sh`
  - `bash scripts/ci/test_ci_tools.sh`
