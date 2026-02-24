# Spec: Issue #5883 - Shell LOC Reduction Wave 2 (Selector Test Surface)

- Issue: #5883
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Shell LOC remains high. `scripts/ci/test_select_targets.sh` has substantial blank-line overhead that can be reduced without altering behavior.

## Scope
In scope:
- Remove redundant blank lines from `scripts/ci/test_select_targets.sh` only.
- Preserve script logic/output semantics.
- Re-validate selector/test contract lanes and shell LOC metrics.

Out of scope:
- Functional changes to selector tests or selectors.

## Acceptance Criteria
### AC-1 Shell LOC decreases
Given pre-change shell metrics,
When post-change metrics are collected,
Then `shell_line_total` is lower.

### AC-2 Selector test behavior is preserved
Given selector matrix regression tests,
When `scripts/ci/test_select_targets.sh` runs,
Then tests pass unchanged.

### AC-3 Review contract lane remains green
Given review docs contract tests,
When `cargo test -p kamn-core --test review_r53_docs_contract` runs,
Then it remains green.

## Conformance Cases
- C-01 (Integration, AC-1): `check_shell_loc_hard_ceiling.sh` reports reduced `shell_line_total`.
- C-02 (Regression, AC-2): `scripts/ci/test_select_targets.sh` passes.
- C-03 (Functional, AC-3): `review_r53_docs_contract` passes.

## Success Metrics / Observable Signals
- `shell_line_total` reduced vs pre-change baseline.
- Selector regression suite exits 0.
- Review contract tests remain passing.
