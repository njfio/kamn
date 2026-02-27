# Spec: Issue #6139 - Stabilize shell-test surface ratio non-regression gate

- Issue: #6139
- Status: Reviewed
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r68-r59-swarm-remediation-and-full-gap-closure/index.md`
- Last Updated: 2026-02-27
- Parent: #6103

## Problem Statement
R59 reported `functional_shell_test_surface_ratio_non_regression_gate` as failing. The fixture states this policy is based on git-tracked files only, but the implementation walked the filesystem directly, allowing untracked local files to perturb counts and create nondeterministic outcomes.

## Scope
In scope:
- Make ratio counting deterministic by deriving shell/rust test-file counts from git-tracked files.
- Add regression coverage proving untracked shell test files are ignored.
- Preserve existing schema/reason-taxonomy/report contracts.

Out of scope:
- Reworking shell-surface policy thresholds.
- Broad shell/rust ratio policy redesign outside this gate implementation.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Surface counting uses only git-tracked files under `scripts/` and `crates/`.
- AC-2: Adding an untracked `scripts/test_*.sh` file does not change evaluated counts/ratio.
- AC-3: Existing shell-test-surface ratio policy test target remains green with deterministic output.

## Conformance Cases
- C-01 (Functional, AC-1): `current_surface()` derives counts from `git ls-files` result set, not recursive filesystem traversal.
- C-02 (Regression, AC-2): `regression_shell_surface_ratio_ignores_untracked_shell_test_files` passes.
- C-03 (Conformance, AC-3): `cargo test -p kamn-core --test shell_test_surface_ratio_policy` passes.

## Success Metrics / Observable Signals
- No shell-test surface ratio drift from untracked workspace artifacts.
- Reported reason-code failures for this gate stop recurring due to local/untracked filesystem contamination.
