# Tasks — Issue #4876

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence: wrapper matrix contract for non-Kolme lightweight waves was ratcheted to include wave-19 coverage and fail-closed unknown-wrapper assertions.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: merged implementation in PR #4892 extended parameterized non-Kolme lightweight wrapper coverage to wave-19 and aligned dispatcher matrix paths.
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: wave-specific duplication remained behind shared runner and definitions, with wave-19 support landing without introducing new bespoke matrix scripts.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `bash scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh` -> passed
    - `bash scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh --wave 19` -> passed
    - `bash scripts/ci/test_ci_tools.sh` -> passed
