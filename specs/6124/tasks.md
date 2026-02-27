# Tasks: Issue #6124

## Ordered Tasks
- [x] T1 (RED/Conformance): Captured RED with a fail-closed budget probe before migration (`docs_contract_test_file_count=74`, `max=62`, fail/NO-GO).
- [x] T2 (Implementation): Removed 12 unreferenced `*_docs.rs` placeholder/stub files and wired a docs-contract test-file budget gate into `ci-fast-gate` (`run_script_surface_budget_checks` scope).
- [x] T3 (GREEN/Regression): Re-ran the budget probe and observed green (`docs_contract_test_file_count=62`, `max=62`, `status=ok`).
- [x] T4 (Verification): Executed `bash scripts/ci/test_workflow_scope_policy.sh`, `bash scripts/ci/test_ci_tools_command_surface_contract.sh`, budget probe command (`git ls-files 'crates/*/tests/*_docs.rs'` vs `.ci/docs-contract-test-file-budget.env`), `bash scripts/ci/test_select_targets.sh`, `bash scripts/ci/test_ci_strategy_contract.sh`, `cargo test -p kamn-core --test docs_contract_wave3_harness -- --nocapture`, `cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture`, `cargo fmt --check`, and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [x] T5 (Closure): Opened PR #6180 with AC->test mapping and RED/GREEN evidence; issue process log updated with measured outputs.

## Tier Mapping
- Unit: T1, T3, T4
- Functional: T3, T4
- Integration: T4 (when cross-module behavior is affected)
- Regression: T1, T3, T4
- Conformance: T1, T4, T5
