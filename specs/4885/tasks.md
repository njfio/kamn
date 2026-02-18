# Tasks — Issue #4885

- [x] T1 (Red): add/update conformance checks that assert ratchet-regression fail behavior and exception-gated pass behavior.
- [x] T2 (Green): implement downward-only ratchet enforcement, strict exception parsing, and deterministic marker output in fast-gate threshold checks.
- [x] T3 (Refactor): centralize ratchet constants/parsing in `fast_gate_budget_delta.py` and reuse across threshold + contract-lane tests.
- [x] T4 (Verify): run required script-level functional/integration/conformance regression lanes and record evidence.

## Verification Evidence

- `bash scripts/ci/test_check_fast_gate_budget_delta_threshold.sh` → `fast-gate budget delta threshold checker tests passed.`
- `bash scripts/ci/test_run_fast_gate_budget_delta_contract_lane.sh` → `Fast-gate budget-delta contract lane tests passed.`
- `bash scripts/ci/test_ci_strategy_contract.sh` → `CI strategy contract tests passed.`
- `bash scripts/ci/test_generate_fast_gate_budget_delta_report.sh` → `fast-gate budget delta report generator tests passed.`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh` → `ci tools command surface contract tests passed.`
- `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh` → `Fast-mode CI tool regression tests passed.`
