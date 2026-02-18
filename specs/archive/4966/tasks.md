# Issue #4966 Tasks

- Issue: #4966
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add failing tests for ratchet regression and waiver-metadata violations.
- [x] T2 (Green): implement threshold-ratchet checker and workflow wiring.
- [x] T3 (Refactor): harden deterministic reason and telemetry outputs.
- [x] T4 (Regression): validate fast-gate wiring and ratchet policy suites.

## Completion Evidence
- Subtask delivery: `#4978` (PR `#4986`, merged)
- `bash scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
- `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
