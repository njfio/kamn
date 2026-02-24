# Spec: Issue #5893 - Branch-Diff Freeze Guard for Existing r51+ Review Docs

- Issue: #5893
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Current review freeze contracts validate manifest content and r57+ history commit-count constraints, but they do not explicitly fail PRs that modify already-tracked frozen review docs in branch diff.

## Scope
In scope:
- Add branch-diff immutability guard in review docs contract tests for existing `docs/review/gaps-and-issues-rNN.md` where `NN >= review_document_freeze_effective_release_min`.
- Enforce that branch diff allows only additions of new review docs; modifications/deletions/renames of existing frozen docs fail.

Out of scope:
- Rewriting historical review docs.
- CI workflow file changes.

## Acceptance Criteria
### AC-1 Branch-diff modification of existing frozen review docs is blocked
Given branch diff `origin/main...HEAD`,
When an existing `r51+` review doc appears with non-add status,
Then docs contract test fails with deterministic message.

### AC-2 New review-doc additions remain permitted
Given branch diff `origin/main...HEAD`,
When a new `rNN` review doc is added,
Then branch-diff guard allows it.

### AC-3 Existing freeze policy assertions remain intact
Given review freeze policy + manifest tests,
When docs contract suite runs,
Then all existing freeze assertions continue to pass.

### AC-4 Target lane stays green
Given test implementation,
When `cargo test -p kamn-core --test review_r53_docs_contract` runs,
Then it passes.

## Conformance Cases
- C-01 (Functional, AC-1): branch-diff parser rejects non-`A` status for existing frozen review docs.
- C-02 (Functional, AC-2): branch-diff parser permits `A` status for new review docs.
- C-03 (Regression, AC-3): existing freeze-policy tests still pass in same lane.
- C-04 (Integration, AC-4): targeted contract lane is green.

## Success Metrics / Observable Signals
- Deterministic branch-diff freeze guard active in `review_r53_docs_contract` lane.
- No regressions in existing review freeze policy checks.
