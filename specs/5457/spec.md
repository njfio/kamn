# Issue #5457 Spec - Branch Hygiene Refresh Wave

- Status: Accepted
- Issue: #5457
- Parent: #3812
- Milestone: None

## Problem Statement
Remote branch inventory remains above the preferred floor due to merged `origin/codex/*` branches left behind after merge completion.

## Scope
In scope:
- Capture pre-cleanup branch count and merged candidate branch list.
- Delete only branches merged into `origin/main`.
- Capture post-cleanup branch count and document deleted branches.

Out of scope:
- Deleting unmerged branches.
- CI/workflow/template modifications.

## Acceptance Criteria
- AC-1: Every deleted branch is verified merged into `origin/main`.
- AC-2: Post-cleanup remote branch count is lower than pre-cleanup count.
- AC-3: Cleanup evidence artifact is committed with pre/post counts and deletion list.

## Conformance Cases
- C-01 (Functional, AC-1): candidate list is generated from `git branch -r --merged origin/main`.
- C-02 (Functional, AC-2): `git ls-remote --heads origin | wc -l` decreases after cleanup.
- C-03 (Conformance, AC-3): `docs/planning/2026-02-21-branch-hygiene-refresh-wave.md` records commands and outcomes.

## Success Metrics / Observable Signals
- Branch count drops from 55 to 50 in this wave.
- Deleted branches are listed and all are merged-only.
