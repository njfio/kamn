# Spec: Issue #5806 - Execute Merged-Lineage Branch Cleanup Tranche (52 -> 50)

- Issue: #5806
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r55-e2e-evidence-step-inventory-parity/index.md`

## Problem Statement
R54 closure markers carry an active branch-budget target requiring remote heads to be reduced to `<=50` by the next release. Current remote count is `52`, so a bounded cleanup tranche is required. The cleanup must be merged-lineage only and must avoid deleting protected heads.

## Acceptance Criteria
- AC-1: Capture pre-cleanup remote branch snapshot and count.
- AC-2: Reconcile exactly 2 merged stale branch refs using merged-lineage-only selection (`main`/HEAD excluded).
- AC-3: Post-cleanup remote branch count is `<=50`.
- AC-4: Lifecycle artifacts and milestone metadata are updated and finalized.
- AC-5: Verification evidence and process log comments are recorded in issue/PR.

## Scope
In scope:
- Remote branch cleanup for 2 merged heads.
- `specs/5806/spec.md`
- `specs/5806/plan.md`
- `specs/5806/tasks.md`
- `specs/milestones/r55-e2e-evidence-step-inventory-parity/index.md`

Out of scope:
- Deleting unmerged or active branches.
- CI/workflow/script changes.
- Runtime/feature code changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | `git branch -r` snapshot | Deterministic pre-cleanup count captured (`52`). |
| C-02 | AC-2 | Functional | merged remote branch list + prune reconciliation | Exactly two stale merged refs are reconciled; `main`/HEAD excluded. |
| C-03 | AC-3 | Conformance | post-delete `git branch -r | wc -l` | Remote branch count is `50` or lower. |
| C-04 | AC-4 | Regression | milestone/spec artifact inspection | `#5806` lifecycle artifacts and milestone delivery slice reflect completion. |
| C-05 | AC-5 | Regression | issue/PR process log comments | Status and verification evidence are posted with deterministic markers. |

## Test Mapping
- `git branch -r | wc -l`
- `git branch -r --merged origin/main`
- `git fetch --prune origin`
- `git branch -r | wc -l` (post)
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics
- Remote branch count reduced from `52` to `50`.
- No protected or unmerged branch deleted.
- `#5806` lifecycle and milestone records finalized with reproducible evidence.
