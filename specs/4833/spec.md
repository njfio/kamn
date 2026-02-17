# Spec — Issue #4833

- Title: Subtask: update AGENTS/CONTRIBUTING and issue templates with shell-surface DoR/DoD gates
- Parent: Parent task: #4818
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Add explicit shell-surface governance intake/closure gates to contributor contracts and issue templates, then enforce them via deterministic docs-contract tests.

## Problem Statement

Process contracts did not require explicit shell-vs-Rust impact fields at issue intake and closure time, allowing script-surface growth to bypass consistent DoR/DoD accounting.

## Scope

In scope:
- add shell-surface DoR/DoD marker blocks to `AGENTS.md`
- add shell-surface DoR/DoD marker blocks to `.github/CONTRIBUTING.md`
- add shell-surface governance estimate fields to all issue templates
- add deterministic docs-contract test for these markers
- wire test into CI tools regression lane

Out of scope:
- PR template enforcement details (handled in `#4834`)
- CI workflow policy threshold changes

## Acceptance Criteria

- AC-1: AGENTS and CONTRIBUTING contracts include explicit shell-surface DoR/DoD marker requirements.
- AC-2: Epic/Story/Task/Subtask issue templates require shell-surface estimate markers for script/workflow/template changes.
- AC-3: CI docs-contract test fails closed when these governance markers drift or are removed.

## Conformance Cases

- C-01 (AC-1, Functional): `bash scripts/ci/test_shell_surface_issue_intake_contract.sh` validates AGENTS/CONTRIBUTING DoR/DoD markers.
- C-02 (AC-2, Conformance): `bash scripts/ci/test_shell_surface_issue_intake_contract.sh` validates required shell-surface fields in all issue templates.
- C-03 (AC-3, Integration/Regression): `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes with shell-surface intake contract test wired in.

## Success Metrics / Signals

- Governance docs and templates expose deterministic shell-surface marker keys.
- Missing marker regressions fail closed in CI tools regression.
- Future intake/closure flow requires explicit shell-vs-Rust delta accounting fields.
