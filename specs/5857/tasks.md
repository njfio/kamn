# Tasks: Issue #5857

- [ ] T1 (Tests first): add regression expectations for missing push-trigger markers in `crates/kamn-core/tests/e2e_live_workflow_lane.rs`.
- [ ] T2 (Tests first): add phase-4i workflow contract assertions for `push` and `main` branch scope in `crates/kamn-e2e-harness/tests/phase4i_ci_workflow_contract.rs`.
- [ ] T3 (Implementation): update `.github/workflows/e2e-live.yml` to include `push` trigger and bounded lane execution policy.
- [ ] T4 (Implementation): sync `docs/ci/strategy.md` E2E live contract marker list with updated reason taxonomy.
- [ ] T5 (Regression): run targeted workflow-contract tests for `kamn-core` and `kamn-e2e-harness`.
- [ ] T6 (Verify): run `cargo fmt --check` and scoped `cargo clippy` checks for touched test crates.

## Verification Evidence (to fill during implementation)
- RED:
  - `cargo test -p kamn-core --test e2e_live_workflow_lane regression_e2e_live_workflow_lane_rejects_missing_push_trigger -- --exact`
- GREEN/Regression:
  - `cargo test -p kamn-core --test e2e_live_workflow_lane`
  - `cargo test -p kamn-e2e-harness --test phase4i_ci_workflow_contract`
- Verify:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core --test e2e_live_workflow_lane -- -D warnings`
  - `cargo clippy -p kamn-e2e-harness --test phase4i_ci_workflow_contract -- -D warnings`
