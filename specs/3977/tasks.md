# Tasks — Issue #3977

- [x] T1 (Red): add failing rustdoc policy tests for runtime-budget status and runtime-budget deterministic reason code.
- [x] T2 (Green): implement runtime-budget marker and reason-code handling in rustdoc artifact policy checker.
- [x] T3 (Refactor): update docs marker strings and CI strategy contract expectations.
- [x] T4 (Verify): run targeted rustdoc/docs contracts and fast CI tools regression.

## Planned Verification Commands

- `bash scripts/ci/test_check_kamn_core_rustdoc_artifact_policy.sh`
- `bash scripts/ci/test_run_kamn_core_rustdoc_artifact_contract_lane.sh`
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
