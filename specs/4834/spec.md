# Spec — Issue #4834

- Title: Subtask: add docs-contract and PR-template enforcement for script LOC delta and ratio trend markers
- Parent: Parent task: #4818
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Enforce shell LOC delta and shell-to-Rust ratio trend declarations in PR process by updating PR template contracts and checker automation.

## Problem Statement

PR CI declaration checks did not enforce shell-surface delta markers, leaving process guardrails incomplete for script-heavy changes.

## Scope

In scope:
- add shell-surface impact declaration section to PR template
- extend `check_pr_ci_declaration.sh` to enforce shell-surface declaration markers when shell-sensitive changes are present
- add deterministic docs-contract test for PR template/checker marker wiring
- extend checker tests to cover shell-sensitive pass/fail paths
- wire docs-contract test into CI tools regression lane
- update CI strategy documentation with shell-surface PR declaration contract

Out of scope:
- CI policy threshold redesign
- non-PR event governance changes

## Acceptance Criteria

- AC-1: PR template includes explicit shell LOC delta / ratio trend declaration markers.
- AC-2: PR declaration checker enforces shell-surface declaration fields and accepted ratio status values for shell-sensitive changes.
- AC-3: Docs-contract and CI regression tests fail closed if PR-template/checker shell-surface markers drift.

## Conformance Cases

- C-01 (AC-1, Functional): `bash scripts/ci/test_pr_template_shell_surface_markers_contract.sh` verifies PR template marker presence.
- C-02 (AC-2, Conformance): `bash scripts/ci/test_check_pr_ci_declaration.sh` verifies shell-sensitive declaration pass/fail behavior and allowed ratio-status values.
- C-03 (AC-3, Integration/Regression): `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes with PR-template marker contract test in fast-mode lane.

## Success Metrics / Signals

- PR template includes deterministic shell-surface declaration marker keys.
- Checker fails closed when shell-sensitive PR declarations are missing/invalid.
- Fast-mode CI regression includes and passes PR-template marker contract coverage.
