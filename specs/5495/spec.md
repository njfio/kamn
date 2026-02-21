# Issue #5495 Spec - Stale Closed-Issue Branch Trim

- Status: Implemented
- Issue: #5495
- Parent: #3812
- Milestone: R50.13 Branch hygiene stale closed-issue branch trim

## Problem Statement
Remote branch heads are currently `51`. Merged-only candidate set is exhausted, but stale closed-issue branches remain and can be trimmed safely using explicit deterministic criteria.

## Scope
In scope:
- Capture pre-cleanup branch count.
- Build deterministic stale candidate set where each branch satisfies:
  1. associated issue is closed,
  2. no open PR uses the branch as head,
  3. branch tip is not merged into `origin/main`.
- Delete exactly one candidate branch.
- Verify post-cleanup branch count is `50`.

Out of scope:
- Deleting more than one branch.
- Deleting branches for open issues or active open PR heads.
- Runtime/product code changes.

## Acceptance Criteria
- AC-1: Pre-cleanup branch count and candidate filter commands are recorded.
- AC-2: Deleted branch satisfies all three stale-safe criteria.
- AC-3: Exactly one branch is deleted and post-cleanup count equals `50`.
- AC-4: Evidence is captured in `specs/5495/evidence.md`.

## Conformance Cases
- C-01 (AC-1): evidence file includes command set and pre-count `51`.
- C-02 (AC-2): evidence file lists deleted branch and confirms closed issue, no open PR, non-merged status.
- C-03 (AC-3): post-count command output equals `50`.
- C-04 (AC-4): evidence artifact committed and referenced in PR.

## Success Metrics / Observable Signals
- Remote branch head count decreases by exactly one.
- No active branch (open issue/open PR) is deleted.
