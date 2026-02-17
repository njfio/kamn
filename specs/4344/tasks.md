# Tasks — #4344

Status: Reviewed

T1 (RED)
- Add failing ratio-governance assertions to rustdoc artifact policy tests (`#4351`).

T2 (GREEN)
- Implement checker/report ratio-governance fields and deterministic reasons (`#4352`).

T3 (Regression)
- Run:
  - `bash scripts/ci/test_check_kamn_core_rustdoc_artifact_policy.sh`
  - `bash scripts/ci/test_run_kamn_core_rustdoc_artifact_contract_lane.sh`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

T4
- PR, CI, merge, and issue closure updates.
