# Spec: #5686 Prune Merged Codex Issue Branches

- Issue: #5686
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P2

## Problem Statement
Merged `codex/issue-*` branches have accumulated on `origin`, increasing branch
inventory and obscuring active branches.

## Scope
### In Scope
- Measure remote branch count baseline.
- Enumerate `origin/codex/issue-*` branches whose tips are reachable from `origin/main`.
- Delete only those merged branches.
- Record before/after branch counts and deletion evidence.

### Out of Scope
- Deleting non-`codex/issue-*` branches.
- Deleting any branch not merged into `origin/main`.
- CI/workflow or repository policy changes.

## Acceptance Criteria
### AC-1 Safe candidate inventory
Given remote branches on `origin`,
When candidates are generated,
Then only `origin/codex/issue-*` branches merged into `origin/main` are listed.

### AC-2 Merged-only pruning
Given candidate branches from AC-1,
When prune operations run,
Then only candidate branches are deleted from `origin`.

### AC-3 Hygiene telemetry
Given cleanup completion,
When verification runs,
Then before/after remote branch counts are recorded in issue/PR evidence.

## Conformance Cases
- C-01 (AC-1): merged-branch filter command returns only `codex/issue-*` names.
- C-02 (AC-2): each delete operation succeeds and no unmerged/non-matching branches are targeted.
- C-03 (AC-3): telemetry includes branch count before and after pruning.

## Success Metrics
- Remote branch count decreases.
- No open PR head branch is deleted.
- `gh issue list --state open` remains unaffected by cleanup.
