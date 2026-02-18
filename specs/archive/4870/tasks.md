# Tasks — Issue #4870

- [x] T1 (Red): conformance tests extended for ratchet-regression fail/exception markers and telemetry threshold-reason drift paths.
- [x] T2 (Green): downward-only ratchet enforcement and deterministic reason-marker contracts wired into fast-gate checks.
- [x] T3 (Refactor): shared deterministic marker/reason-taxonomy behavior consolidated across threshold checker + contract lanes.
- [x] T4 (Verify): targeted + fast-mode integration regressions run; parent task rolled up from merged subtasks.

## Verification Evidence

- Subtask delivery PRs: `#4896` (ratchet policy/wiring), `#4897` (telemetry/policy deterministic conformance expansion).
- `bash scripts/ci/test_check_fast_gate_budget_delta_threshold.sh` → `fast-gate budget delta threshold checker tests passed.`
- `bash scripts/ci/test_run_fast_gate_budget_delta_contract_lane.sh` → `Fast-gate budget-delta contract lane tests passed.`
- `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh` → `shell-rust LOC telemetry collector tests passed.`
- `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh` → `combined shell-surface trend policy checker tests passed.`
- `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh` → `Fast-mode CI tool regression tests passed.`
