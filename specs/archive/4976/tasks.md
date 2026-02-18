# Issue #4976 Tasks

- Issue: #4976
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): capture hard-ceiling failure path from an intentionally low threshold.
  Evidence:
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh --ceiling-file <tmp HARD_SHELL_LOC_MAX=30000> --output-json <tmp>` -> exit `1`, `reason_codes=shell_loc_hard_ceiling_exceeded`
- [x] T2 (Green): validate pass path using canonical fixture.
  Evidence:
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh --ceiling-file .ci/shell-loc-hard-ceiling.env --output-json <tmp>` -> exit `0`
- [x] T3 (Refactor): avoid non-issue churn and keep closure scoped.
  Evidence:
  - Only `specs/4976/*` lifecycle artifacts updated in closure commit.
- [x] T4 (Regression): execute checker + wiring regression tests.
  Evidence:
  - `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- [x] T5 (Docs): synchronize issue process markers for shell-surface DoR and phase updates.
  Evidence:
  - Issue body updated with shell-surface estimate markers.
  - InProgress lifecycle comment posted on issue `#4976`.
- [x] T6 (Verify): finalize conformance mappings and lifecycle status.
  Evidence:
  - `specs/4976/spec.md`, `specs/4976/plan.md`, `specs/4976/tasks.md` set to Implemented.

## Completion Evidence
- Hard ceiling checker fixture and red/green behavior are verified and lifecycle artifacts are synchronized to implemented state.
