# Plan — Issue #4833

## Approach

1. Add a docs-contract test that fails when shell-surface DoR/DoD markers are missing from AGENTS/CONTRIBUTING/templates.
2. Add deterministic marker blocks to:
   - `AGENTS.md`
   - `.github/CONTRIBUTING.md`
   - `.github/ISSUE_TEMPLATE/{epic,story,task,subtask}.md`
3. Wire the new docs-contract test into `scripts/ci/test_ci_tools.sh` (fast/full blocks).
4. Re-run targeted contract tests and CI tools fast-mode regression.

## Affected Modules

- `AGENTS.md`
- `.github/CONTRIBUTING.md`
- `.github/ISSUE_TEMPLATE/epic.md`
- `.github/ISSUE_TEMPLATE/story.md`
- `.github/ISSUE_TEMPLATE/task.md`
- `.github/ISSUE_TEMPLATE/subtask.md`
- `scripts/ci/test_shell_surface_issue_intake_contract.sh`
- `scripts/ci/test_ci_tools.sh`

## Risks / Mitigations

- Risk: governance marker drift between docs and templates.
  Mitigation: one contract test checks all required marker keys in all required files.
- Risk: CI regression lane noise from brittle wording checks.
  Mitigation: assert stable marker keys instead of prose phrasing.
- Risk: process overhead for non-script work.
  Mitigation: markers are required only when script/workflow/template surface changes.

## Interfaces / Contracts

- Intake marker contract:
  - `shell_loc_delta_estimate`
  - `rust_loc_delta_estimate`
  - `shell_to_rust_ratio_delta_estimate`
  - `shell_surface_mitigation_issue`
- Closure marker contract:
  - `shell_loc_delta_actual`
  - `rust_loc_delta_actual`
  - `shell_to_rust_ratio_delta_actual`
  - `shell_surface_ratio_target_status`

## ADR

No ADR required. No protocol/dependency boundary changes were introduced.
