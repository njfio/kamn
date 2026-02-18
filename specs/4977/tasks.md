# Issue #4977 Tasks

- Issue: #4977
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): capture failing behavior for threshold violations.
  Evidence:
  - `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file <tmp WARN=0.10 FAIL=0.20> --output-json <tmp>` -> exit `1`, `reason_codes=shell_rust_ratio_fail_threshold_exceeded`
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh --ceiling-file <tmp HARD_SHELL_LOC_MAX=30000> --output-json <tmp>` -> exit `1`, `reason_codes=shell_loc_hard_ceiling_exceeded`
- [x] T2 (Green): verify current fast-gate threshold configuration passes checker gates.
  Evidence:
  - `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file .ci/shell-rust-ratio-guardrail.env --output-json <tmp>` -> exit `0`
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh --ceiling-file .ci/shell-loc-hard-ceiling.env --output-json <tmp>` -> exit `0`
- [x] T3 (Refactor): keep scope minimal and avoid non-issue code churn.
  Evidence:
  - Closure commit only updates `specs/4977/*` lifecycle artifacts.
- [x] T4 (Regression): run wiring + checker contract regression suite.
  Evidence:
  - `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
  - `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- [x] T5 (Docs): synchronize issue/process markers for shell-surface DoR and phase tracking.
  Evidence:
  - Updated issue body with shell-surface estimate markers.
  - Added process log comments on issue `#4977`.
- [x] T6 (Verify): map ACs to conformance and finalize spec lifecycle status.
  Evidence:
  - `specs/4977/spec.md` AC/test mapping updated.
  - `specs/4977/plan.md` and `specs/4977/tasks.md` marked Implemented.

## Completion Evidence
- Fast-gate required shell ratio + hard-ceiling checks are verified integrated and fail-closed behaviors are evidenced with deterministic outputs.
