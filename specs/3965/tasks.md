# Tasks - Issue #3965

- [x] T1 (Red): identify missing story lifecycle artifacts and AC mapping gaps.
- [x] T2 (Green): deliver missing-docs graduation and navigation/rustdoc governance via child tasks `#3968` and `#3969`.
- [x] T3 (Refactor/Docs): codify story-level docs governance conformance mapping.
- [x] T4 (Verify): run missing-docs policy, rustdoc/navigation checks, strategy contracts, and fast-gate integration checks.

## Planned Verification Commands

- `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- `bash scripts/ci/test_missing_docs_graduation_batch_report_contract.sh`
- `bash scripts/ci/test_run_kamn_core_rustdoc_artifact_contract_lane.sh`
- `bash scripts/ci/test_check_kamn_core_rustdoc_artifact_policy.sh`
- `cargo test -p kamn-core --test runtime_architecture_docs`
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
