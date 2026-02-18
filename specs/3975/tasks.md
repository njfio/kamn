# Tasks — Issue #3975

- [x] T1 (Red): add failing tests asserting deterministic reason markers for graduated-module exemption regression and updated reason-code CSV.
- [x] T2 (Green): implement exemption-regression reason-marker emission in missing-docs policy checker.
- [x] T3 (Refactor): synchronize docs marker references and docs-contract assertions.
- [x] T4 (Verify): run targeted missing-docs/docs tests and fast CI tools regression.

## Planned Verification Commands

- `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- `cargo test -p kamn-core runtime_architecture_docs`
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
