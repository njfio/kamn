# Tasks: Issue #6124

## Ordered Tasks
- [x] T1 (RED/Conformance): Added docs-contract test-file budget checker and captured RED via `bash scripts/ci/check_docs_contract_test_file_budget.sh --threshold-file .ci/docs-contract-test-file-budget.env --output-json /tmp/docs-contract-test-file-budget-red.json` (`docs_contract_test_file_count=74`, `max=62`, fail-closed).
- [x] T2 (Implementation): Removed 12 unreferenced `*_docs.rs` placeholder/stub files and wired docs-contract file-budget checks into CI fast-gate + CI tools regression.
- [x] T3 (GREEN/Regression): Re-ran docs-contract file-budget checker and observed green (`docs_contract_test_file_count=62`, `max=62`, `status=ok`).
- [x] T4 (Verification): Executed `bash scripts/ci/test_check_docs_contract_test_file_budget.sh`, `bash scripts/ci/test_workflow_scope_policy.sh`, `bash scripts/ci/test_ci_tools_command_surface_contract.sh`, `bash scripts/ci/test_select_targets.sh`, `bash scripts/ci/test_ci_strategy_contract.sh`, `cargo test -p kamn-core --test docs_contract_wave3_harness -- --nocapture`, `cargo test -p kamn-core --test shell_test_surface_ratio_policy -- --nocapture`, `cargo fmt --check`, and `cargo clippy -p kamn-core --tests -- -D warnings`.
- [ ] T5 (Closure): Open PR with AC->test mapping and RED/GREEN evidence; close issue after merge with measured outputs.

## Tier Mapping
- Unit: T1, T3, T4
- Functional: T3, T4
- Integration: T4 (when cross-module behavior is affected)
- Regression: T1, T3, T4
- Conformance: T1, T4, T5
