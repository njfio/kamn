# Spec: Issue #5881 - Shell LOC Reduction Wave (No Behavior Change)

- Issue: #5881
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Shell-surface size remains high and is repeatedly flagged in audits. A low-risk reduction pass is needed to shrink tracked shell LOC while preserving behavior.

## Scope
In scope:
- Reduce tracked shell LOC in a high-volume CI script by removing redundant blank-line overhead only.
- Preserve all script logic, variable names, and control flow behavior.
- Validate with existing selector regression tests and shell-LOC policy checks.

Out of scope:
- Functional changes to CI scope-selection semantics.
- Refactoring command logic or altering output keys.

## Acceptance Criteria
### AC-1 Shell LOC decreases after reduction pass
Given tracked shell baseline at issue start,
When post-change shell metrics are collected,
Then `shell_line_total` is lower than the baseline.

### AC-2 Selector behavior remains unchanged
Given selector contract tests,
When `scripts/ci/test_select_targets.sh` executes,
Then all assertions pass.

### AC-3 Shell policy contracts remain green
Given shell LOC and docs contracts,
When shell/doc contract checks execute,
Then no ratchet/contract failure is introduced.

## Conformance Cases
- C-01 (Integration, AC-1): `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json ...` shows reduced `shell_line_total`.
- C-02 (Regression, AC-2): `scripts/ci/test_select_targets.sh` passes.
- C-03 (Functional, AC-3): `cargo test -p kamn-core --test review_r53_docs_contract` remains green.

## Success Metrics / Observable Signals
- `shell_line_total` decreases from pre-change baseline.
- `scripts/ci/test_select_targets.sh` exits 0.
- `review_r53_docs_contract` passes, including shell LOC ratchet assertions.
