# 7052-restore-local-workspace-gate-baselines

## Objective
Restore local workspace gate baselines that drifted after the latest MVP proof
merge train without weakening any gate, threshold, classifier, or proof
semantics.

## Inputs/Outputs
- Inputs:
  - Latest `main` head after PR #7050.
  - `crates/kamn-core/tests/test_file_size_policy.rs`.
  - `fixtures/ci/test_file_size_policy_baseline.env`.
  - `crates/kamn-core/tests/governance_feature_commit_ratio_base_compliance/current_head_status_contract_tests.rs`.
  - `scripts/ci/check_governance_feature_commit_ratio.py`.
  - `README.md`.
  - `crates/kamn-e2e-harness/tests/readme_mvp_front_door_contract.rs`.
- Outputs:
  - Test file-size inventory baseline matches the current tracked test-file
    inventory.
  - Current-head governance ratio contract matches the real compliant 50-commit
    checker output.
  - README front-door contract headers match both the migrated compact README
    lane and the evaluator-facing MVP front-door lane.
  - The #7051 branch remains governance-ratio compliant.

## Boundaries/Non-goals
- Do not change policy thresholds, classifier behavior, CI workflows, formatter,
  clippy, or proof semantics.
- Do not add filler runtime behavior or unrelated feature surface.
- Do not change KAMN demo claim boundaries.
- Do not use this issue to relax any failing test.

## Failure Modes
- The baseline fixture is refreshed from an untracked or generated file
  inventory instead of tracked `crates/**/tests/**` Rust files.
- The exact governance expectation is updated without verifying the checker
  output.
- The #7051 branch remains above the 0.20 governance-ratio cap.
- A history repair loses the #7051 issue/spec/TDD evidence.
- README content remains useful but drifts from either the contracted compact
  onboarding section names or the evaluator-facing MVP front-door names.

## Acceptance Criteria
- [ ] Existing red gate evidence reproduces the test file inventory drift.
- [ ] Existing red gate evidence reproduces the stale current-head governance
      expectation.
- [ ] `test_file_total` equals the tracked test-file inventory.
- [ ] The current-head governance contract expects 10 governance, 40 feature,
      ratio 0.2, and feature ratio 0.8 for the current compliant window.
- [ ] README exposes the contracted onboarding headers while staying within the
      200-line cap.
- [ ] README exposes the evaluator-facing MVP front-door headers in order:
      current proof, quickstart, claim boundaries, repository map, and
      maintainer/agent workflow.
- [ ] The governance-ratio checker reports `status=ok` at the repaired #7051
      branch head.
- [ ] Targeted `kamn-core` gate tests pass.

## Files To Touch
- `specs/7052-restore-local-workspace-gate-baselines.md`
- `fixtures/ci/test_file_size_policy_baseline.env`
- `crates/kamn-core/tests/governance_feature_commit_ratio_base_compliance/current_head_status_contract_tests.rs`
- `README.md`

## Error Semantics
- Baseline drift remains a hard test failure.
- Governance ratio above 0.20 remains a hard checker failure.
- Missing, malformed, or stale baseline values fail loudly through existing
  tests.

## Test Plan
- Red:
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p kamn-core --test test_file_size_policy spec_c04_oversized_test_counts_are_within_budget -- --exact --nocapture`
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p kamn-core --test governance_feature_commit_ratio_base_compliance current_head_status_contract_tests::current_branch_head_restores_ratio_compliance -- --exact --nocapture`
  - `CARGO_TARGET_DIR=target/mvp-demo-proof CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0 cargo test -p kamn-e2e-harness --test readme_mvp_front_door_contract -- --nocapture`
- Green:
  - Refresh only the baseline fixture, exact current-head expectations, and
    README section names.
- Verify:
  - Rerun both targeted red commands.
  - Rerun `readme_compact_contract`, `readme_contract_lane`, and
    `readme_mvp_front_door_contract`.
  - Run the governance-ratio checker against the repaired branch head.
