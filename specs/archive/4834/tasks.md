# Tasks — Issue #4834

- [x] T1 (Red): add failing docs-contract test for PR-template shell-surface markers.
  Evidence:
  - `bash scripts/ci/test_pr_template_shell_surface_markers_contract.sh` failed with:
    `expected PR template shell-surface declaration section marker '## Shell-Surface Impact Declaration' in .../.github/pull_request_template.md`
- [x] T2 (Green): add PR-template shell-surface declaration markers and checker enforcement.
  Evidence:
  - Updated `.github/pull_request_template.md` with shell-surface declaration fields.
  - Extended `scripts/ci/check_pr_ci_declaration.sh` with shell-sensitive enforcement and accepted ratio status validation.
- [x] T3 (Refactor): add docs-contract and checker test coverage for shell-sensitive paths.
  Evidence:
  - Added `scripts/ci/test_pr_template_shell_surface_markers_contract.sh`.
  - Extended `scripts/ci/test_check_pr_ci_declaration.sh` with shell-sensitive pass/fail cases.
  - Added new docs-contract test to `scripts/ci/test_ci_tools.sh`.
- [x] T4 (Verify): run deterministic suites and fast-mode regression.
  Evidence:
  - `bash scripts/ci/test_pr_template_shell_surface_markers_contract.sh`
  - `bash scripts/ci/test_check_pr_ci_declaration.sh`
  - `bash scripts/ci/test_shell_surface_issue_intake_contract.sh`
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`
