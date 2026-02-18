# Tasks — Issue #4871

- [x] T1 (Red): docs/contract tests expanded to fail when shell governance fields are missing from issue templates and PR declarations.
- [x] T2 (Green): AGENTS + templates + PR declaration checks updated to enforce mandatory shell-surface DoR/DoD markers.
- [x] T3 (Refactor): declaration parsing hardened to require field keys at column 1 and deterministic numeric/status validation.
- [x] T4 (Verify): contract tests and fast-mode CI regressions executed; parent task rolled up from merged subtasks.

## Verification Evidence

- Subtask delivery PRs: `#4894` (PR declaration/docs contract wiring), `#4895` (issue template + AGENTS shell DoR/DoD enforcement).
- `bash scripts/ci/test_check_pr_ci_declaration.sh` → `check_pr_ci_declaration tests passed.`
- `bash scripts/ci/test_pr_template_shell_surface_markers_contract.sh` → `PR template shell-surface marker contract tests passed.`
- `bash scripts/ci/test_shell_surface_issue_intake_contract.sh` → `shell-surface issue intake contract tests passed.`
- `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh` → `Fast-mode CI tool regression tests passed.`
