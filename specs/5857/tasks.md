# Tasks: Issue #5857

- [x] T1 (Tests first): added regression expectations for missing push-trigger markers in `crates/kamn-core/tests/e2e_live_workflow_lane.rs`.
- [x] T2 (Tests first): added phase-4i workflow contract assertions for `push` and `main` branch scope in `crates/kamn-e2e-harness/tests/phase4i_ci_workflow_contract.rs`.
- [x] T3 (Implementation): updated `.github/workflows/e2e-live.yml` to include `push` trigger and bounded lane execution policy.
- [x] T4 (Implementation): synced `docs/ci/strategy.md` E2E live contract marker list with updated reason taxonomy.
- [x] T5 (Regression): ran targeted workflow-contract tests for `kamn-core` and `kamn-e2e-harness`.
- [x] T6 (Verify): ran `cargo fmt --check` and scoped `cargo clippy` checks for touched test crates.

## Verification Evidence (to fill during implementation)
- RED:
  - `cargo test -p kamn-e2e-harness --test phase4i_ci_workflow_contract spec_c01_workflow_contains_required_triggers -- --exact` -> failed before workflow change with `assertion failed: workflow.contains(\"push:\")`.
- GREEN/Regression:
  - `cargo test -p kamn-e2e-harness --test phase4i_ci_workflow_contract spec_c01_workflow_contains_required_triggers -- --exact`
  - `cargo test -p kamn-core --test e2e_live_workflow_lane`
  - `cargo test -p kamn-e2e-harness --test phase4i_ci_workflow_contract`
- Verify:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core --test e2e_live_workflow_lane -- -D warnings`
  - `cargo clippy -p kamn-e2e-harness --test phase4i_ci_workflow_contract -- -D warnings`
