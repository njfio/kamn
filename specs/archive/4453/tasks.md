# Tasks: Issue #4453

Status: Completed
Issue: #4453

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Extend `scripts/ci/test_check_test_harness_loc_soft_budget.sh` with failing assertions for
  normalized reason taxonomy markers and reason-class output.
- Run:
  - `bash scripts/ci/test_check_test_harness_loc_soft_budget.sh`
- Expect RED before checker implementation updates.

T2 (RED, Integration/Conformance):
- Extend `scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh` with failing
  assertions for bounded CI-smoke markers in lane stdout/report payload.
- Run:
  - `bash scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh`
- Expect RED before contract-lane implementation updates.

T3 (GREEN, Implementation):
- Implement deterministic reason-taxonomy constants/normalization in
  `scripts/ci/check_test_harness_loc_soft_budget.py`.
- Implement deterministic bounded CI-smoke markers in
  `scripts/ci/test_harness_loc_soft_budget_contract_lane_impl.sh`.

T4 (GREEN, Docs/Regression):
- Update `docs/ci/strategy.md` with taxonomy and CI-smoke marker references.
- Add docs contract checks in `crates/kamn-core/tests/ci_strategy_docs.rs`.
- Run:
  - `cargo test -p kamn-core --test ci_strategy_docs`

T5 (Verify, Regression):
- Re-run targeted tests and scoped hygiene:
  - `bash scripts/ci/test_check_test_harness_loc_soft_budget.sh`
  - `bash scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh`
  - `cargo test -p kamn-core --test ci_strategy_docs`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`
  - `cargo clippy -p kamn-node -- -D warnings`

## TDD Evidence

- RED command/output:
  - `bash scripts/ci/test_check_test_harness_loc_soft_budget.sh`
    - Failed with: `expected deterministic reason taxonomy version marker for within-budget soft checker path`
  - `bash scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh`
    - Failed with: `expected low-cost CI smoke lane marker from contract lane`
  - `cargo test -p kamn-core --test ci_strategy_docs`
    - Failed with:
      - `assertion failed: DOC.contains(\"test_harness_loc_soft_budget_reason_taxonomy_version=kamn.ci.test-harness-loc-soft-budget-reason-taxonomy.v1\")`
- GREEN command/output:
  - `bash scripts/ci/test_check_test_harness_loc_soft_budget.sh`
    - Passed: `test harness LOC soft budget checker tests passed.`
  - `bash scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh`
    - Passed: `Generic test harness LOC soft-budget contract lane tests passed.`
  - `cargo test -p kamn-core --test ci_strategy_docs`
    - Passed: `23 passed; 0 failed`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-core -- -D warnings`
    - Passed
  - `cargo clippy -p kamn-node -- -D warnings`
    - Passed
- Regression summary:
  - Generic structural-budget checker now emits deterministic taxonomy and normalized reason-value/class markers.
  - Generic CI soft-budget contract lane now emits explicit low-cost CI-smoke boundary markers and deterministic reason key.
  - CI strategy docs and docs-contract tests now fail closed on taxonomy/boundary marker drift.
