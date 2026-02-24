# Tasks: Issue #5850

- [x] T1 (Tests first): Added `crates/kamn-core/tests/e2e_live_workflow_lane.rs` with pass/fail fixture coverage and deterministic reason-code assertions.
- [x] T2 (Implementation): Removed shell/Python checker pair and migrated equivalent assertions into Rust lane.
- [x] T3 (Integration): Updated `scripts/ci/test_ci_tools.sh` and `scripts/ci/test_ci_tools_command_surface_contract.sh` to invoke Rust lane.
- [x] T4 (Docs): Updated `docs/ci/strategy.md` markers to reference Rust lane invocation.
- [x] T5 (Verify): Ran targeted Rust lane and full fast CI-tools regression suite.

## Verification Evidence

- `cargo test -p kamn-core --test e2e_live_workflow_lane`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
