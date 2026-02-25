# Spec: Issue #5988 - Reduce governance coupling and duplicate lifecycle contracts

- Issue: #5988
- Status: Reviewed (agent-authored P1; implementation proceeding with explicit user approval)
- Type: story
- Priority: P1
- Area: governance
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25
- Parent: #5917

## Problem Statement
Governance contract tests under `crates/kamn-core/tests/review_r*.rs` duplicate parsing and repository helper logic across files, increasing maintenance and coupling cost for each governance change.

## Scope
In scope:
- Extract shared review-doc parsing/repository helpers into one shared test helper module.
- Rewire duplicate review contract suites to consume shared helpers.
- Preserve all existing governance invariant checks and reasoned assertions.

Out of scope:
- Removing governance contract coverage.
- Changing review marker schema semantics.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Duplicate `parse_marker_*` and `repo_root` helper logic is centralized in one shared helper module used by multiple `review_r*` tests.
- AC-2: Rewired review contracts keep existing behavior and assertions unchanged.
- AC-3: Targeted governance contract tests pass after consolidation.

## Conformance Cases
- C-01 (Unit, AC-1): Shared helper module exposes marker parsing and repo-root helpers used by at least 6 review contract files.
- C-02 (Functional, AC-2): Existing review contract marker assertions remain present and pass unchanged.
- C-03 (Regression, AC-3): Targeted `review_r*` tests run green after consolidation.

## Success Metrics / Observable Signals
- Duplicate helper definitions removed from affected review contract files.
- Targeted test selectors pass for touched review contract files.
- No CI drift in governance contract behavior for the touched suites.
