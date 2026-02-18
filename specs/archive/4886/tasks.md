# Tasks — Issue #4886

- [x] T1 (Red): add failing checks for shell-surface DoD fields in templates.
  - Updated `scripts/ci/test_shell_surface_issue_intake_contract.sh` to require DoD markers in issue templates.
  - Red evidence: `bash scripts/ci/test_shell_surface_issue_intake_contract.sh` failed with missing `shell_loc_delta_actual` marker in `epic.md`.
- [x] T2 (Green): implement minimal template updates for DoR+DoD parity.
  - Added shell-surface closure markers to:
    - `.github/ISSUE_TEMPLATE/epic.md`
    - `.github/ISSUE_TEMPLATE/story.md`
    - `.github/ISSUE_TEMPLATE/task.md`
    - `.github/ISSUE_TEMPLATE/subtask.md`
- [x] T3 (Refactor): preserve single deterministic contract surface.
  - Reused existing `test_shell_surface_issue_intake_contract.sh` instead of introducing a parallel checker.
  - Kept CI command surface stable by updating marker expectations in-place.
- [x] T4 (Verify): run required test tiers and capture regression evidence.
  - `bash scripts/ci/test_shell_surface_issue_intake_contract.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
  - `bash scripts/ci/test_ci_strategy_contract.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh`
