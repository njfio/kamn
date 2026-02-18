# Issue #4978 Tasks

- Issue: #4978
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add failing tests derived from issue ACs and conformance cases.
  Evidence:
  - `python3 scripts/ci/check_shell_surface_threshold_ratchet.py ... --baseline-hard-ceiling-file <tmp:129999> ... --output-json <tmp>` exited non-zero with:
    - `status=fail`
    - `threshold_ratchet_status=regressed`
    - `threshold_ratchet_violations=HARD_SHELL_LOC_MAX`
- [x] T2 (Green): implement minimum change to satisfy tests deterministically.
  Evidence:
  - Added checker + wrapper + exception template:
    - `scripts/ci/check_shell_surface_threshold_ratchet.py`
    - `scripts/ci/check_shell_surface_threshold_ratchet.sh`
    - `.ci/shell-surface-threshold-ratchet-exception.json`
  - Added dedicated tests:
    - `scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
- [x] T3 (Refactor): simplify and harden without changing behavior.
  Evidence:
  - Consolidated output markers and JSON report schema:
    - `kamn.ci.shell-surface-threshold-ratchet-report.v1`
  - Added explicit reason taxonomy signaling for invalid args/schema/order/waiver paths.
- [x] T4 (Regression): add drift/tamper/marker parity regression checks.
  Evidence:
  - `bash scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- [x] T5 (Docs): update required docs/process markers for issue #4978.
  Evidence:
  - Updated issue #4978 body with shell-surface DoR estimates.
  - Added process log comment on issue #4978 for phase/status tracking.
- [x] T6 (Verify): run scoped unit/functional/integration/regression checks and record evidence.
  Evidence:
  - `python3 -m py_compile scripts/ci/check_shell_surface_threshold_ratchet.py`
  - `bash scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` (exit 0)

## Completion Evidence
- All scoped checks passed and CI workflow wiring now enforces threshold-ratchet policy with JSON artifact output.
