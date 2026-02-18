# Tasks — Issue #4877

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence: Kolme wave budget trend contracts were tightened to assert shared-runner behavior and deterministic policy markers across target wave families.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: merged implementation in PR #4893 introduced shared checker flow and converted duplicated wave entrypoints to symlink-backed runner usage.
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: wave 8/10/11 trend checkers now route through one implementation path with stable telemetry/reason-taxonomy output contracts.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `bash scripts/ci/test_kolme_wave_budget_trend_runner_contract.sh` -> passed
    - `bash scripts/ci/test_check_kolme_wrapper_budget_trend.sh` -> passed
    - `bash scripts/ci/test_ci_tools.sh` -> passed
