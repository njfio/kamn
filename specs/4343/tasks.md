# Tasks — #4343

Status: Reviewed

T1 (RED)
- Add failing assertions in `scripts/ci/test_check_kamn_core_missing_docs_policy.sh` for deterministic allowlist delta evidence markers on pass and stagnation-fail cases.

T2 (GREEN)
- Implement marker emission in `scripts/ci/check_kamn_core_missing_docs_policy.sh`.

T3 (Regression)
- Run:
  - `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
  - `bash scripts/ci/test_missing_docs_velocity_guard_contract.sh`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-core -- -D warnings`

T4
- Update docs/spec references and prepare PR with AC mapping.
