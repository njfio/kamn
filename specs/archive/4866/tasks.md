# Tasks — Issue #4866

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence: conformance contracts for non-Kolme lightweight matrix waves and Kolme wave budget trend runners were ratcheted and required before consolidation completion.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: merged subtasks #4876/#4877 completed parameterized runner rollout for targeted wave/matrix families and removed duplicated checker entrypoint logic.
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: shared runners and definition files now back targeted wave families, including wave-19 non-Kolme coverage and shared Kolme trend checker flow.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `bash scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh` -> passed
    - `bash scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh --wave 19` -> passed
    - `bash scripts/ci/test_kolme_wave_budget_trend_runner_contract.sh` -> passed
    - `bash scripts/ci/test_check_kolme_wrapper_budget_trend.sh` -> passed
