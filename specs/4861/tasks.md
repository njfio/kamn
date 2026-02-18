# Tasks — Issue #4861

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence: phase 3-5 migration contracts were ratcheted in child tasks to fail closed on wrapper drift, wave-runner parity drift, and helper-adoption regressions.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: child tasks #4866 and #4867 delivered definition-driven wave/matrix runners and shared harness/JSON helper migration waves.
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: duplicated wave/matrix scripts and ad-hoc test/json boilerplate were replaced by shared runner/helper patterns with deterministic governance checks.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `bash scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh` -> passed
    - `bash scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh --wave 19` -> passed
    - `bash scripts/ci/test_kolme_wave_budget_trend_runner_contract.sh` -> passed
    - `bash scripts/lib/test_test_harness_migration_contract.sh` -> passed
    - `bash scripts/lib/test_json_write_helper_migration_contract.sh` -> passed
