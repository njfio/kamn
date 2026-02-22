# Spec: Issue #5795 — Execute Merged-Only Remote Branch Cleanup to <=50 Heads

- Issue: #5795
- Milestone: `r52-e2e-live-runtime-integration-hardening`
- Priority: P1
- Status: Implemented
- Last Updated: 2026-02-22

## Problem Statement
Remote branch-head count remains above the R55 hygiene target, continuing a growth pattern called out by R54. Cleanup must be executed safely (merged-only) and evidenced deterministically.

## Scope
- Measure remote head baseline.
- Compute merged-only deletion candidates with safety exclusions.
- Delete enough merged remote branches to reach <=50 heads.
- Capture deterministic evidence (before/after count + deleted list).
- Finalize lifecycle artifacts and milestone slice metadata.

## Out of Scope
- Deleting unmerged branches.
- Runtime/API code changes.
- CI/workflow/template/script changes.

## Acceptance Criteria

### AC-1: Target achieved
Given baseline remote head count,
When merged-only cleanup is executed,
Then resulting remote head count is <=50.

### AC-2: Safe deletion policy enforced
Given cleanup candidate generation,
When branch list is filtered,
Then only branches with merged PR lineage (or direct merge-ancestor lineage) are deleted and protected/open-PR heads are excluded.

### AC-3: Deterministic evidence captured
Given cleanup execution,
When validation completes,
Then issue/process logs contain baseline count, deleted branch list, and final count.

### AC-4: Lifecycle and milestone metadata finalized
Given completed cleanup,
When closure is performed,
Then spec/tasks status are finalized and milestone index includes this completed slice.

## Conformance Cases

| ID | AC | Tier | Case |
|---|---|---|---|
| C-01 | AC-1 | Functional | `git ls-remote --heads origin | wc -l` returns <=50 after cleanup. |
| C-02 | AC-2 | Regression | Each deleted branch has merged PR lineage (or tip ancestor of `origin/main`); protected/open-PR heads are excluded. |
| C-03 | AC-3 | Integration | Issue comments + lifecycle artifacts include before/deleted/after evidence. |
| C-04 | AC-4 | Functional | `specs/5795/spec.md`=Implemented, `specs/5795/tasks.md`=Completed, milestone slice updated. |

## Success Metrics / Observable Signals
- Remote head count reduced to target threshold or lower.
- Zero accidental deletion of protected/unmerged/open-PR branches.
- Lifecycle artifacts and closure evidence complete.
