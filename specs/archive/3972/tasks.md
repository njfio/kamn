# Tasks — Issue #3972

- [x] T1 (Red): add failing tests for decline-window warn/fail, stale threshold metadata, and invalid date handling.
- [x] T2 (Green): implement decline-window and staleness policy logic in combined trend checker.
- [x] T3 (Refactor): align deterministic reason taxonomy/csv and threshold fixture metadata.
- [x] T4 (Verify): run targeted contracts and fast CI tools regression lane.

## Planned Verification Commands

- `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh`
- `bash scripts/ci/test_ci_strategy_contract.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
