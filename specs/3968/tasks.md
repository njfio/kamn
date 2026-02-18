# Tasks - Issue #3968

- [x] T1 (Red): identify missing parent task lifecycle artifacts and AC mapping gaps.
- [x] T2 (Green): complete missing-docs graduation and exemption-regression contracts via `#3974` and `#3975`.
- [x] T3 (Refactor/Docs): codify contributor-facing docs/strategy contract mapping at task level.
- [x] T4 (Verify): run missing-docs policy, throughput/velocity/batch, docs contract, and fast-gate integration checks.

## Planned Verification Commands

- `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- `bash scripts/ci/test_missing_docs_graduation_batch_report_contract.sh`
- `bash scripts/ci/test_missing_docs_velocity_guard_contract.sh`
- `bash scripts/ci/test_missing_docs_throughput_report_contract.sh`
- `cargo test -p kamn-core --test runtime_architecture_docs`
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
