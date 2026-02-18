# Tasks — Issue #3973

- [x] T1 (Red): add failing checker and wiring tests for shell-rust ratio guardrail.
- [x] T2 (Green): implement checker/config and integrate into CI-fast + CI tools lanes.
- [x] T3 (Refactor): align reason taxonomy markers and command-surface contracts.
- [x] T4 (Verify): run targeted tests and fast CI tools regression lane.

## Planned Verification Commands

- `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
- `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
