# Spec: Issue #5802 - Execute Merged-Lineage Branch Cleanup Tranche to Restore <=50 Remote Heads

- Issue: #5802
- Status: Implemented (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Problem Statement
Branch-hygiene contract markers set a target of at most 50 remote heads for R55. Current remote branch count is 51, so one merged-only cleanup tranche is required to restore the target envelope without deleting non-merged work.

## Acceptance Criteria
- AC-1: Baseline branch count is captured deterministically before deletion.
- AC-2: Deletion is restricted to branch(es) fully merged into `origin/main`.
- AC-3: Post-cleanup branch count is `<=50` and count-delta math is consistent.
- AC-4: Lifecycle artifacts and milestone metadata are finalized for #5802.

## Scope
In scope:
- `specs/5802/spec.md`
- `specs/5802/plan.md`
- `specs/5802/tasks.md`
- `docs/research/branch-hygiene-5802-cleanup-evidence.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

Out of scope:
- Non-merged branch deletion.
- Runtime/feature code changes.
- CI/workflow modifications.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `gh api repos/njfio/kamn/branches --paginate` count command | Baseline remote branch count is recorded in evidence artifact. |
| C-02 | AC-2 | Conformance | `git branch -r --merged origin/main` + selected branch deletion | Deleted branch appears in merged-lineage set before deletion. |
| C-03 | AC-3 | Regression | Pre/post count comparison | `post_count <= 50` and `pre_count - post_count = deleted_count`. |
| C-04 | AC-4 | Conformance | Lifecycle files + milestone update | #5802 spec/tasks statuses finalized and milestone completed list updated. |

## Test Mapping
- `gh api repos/njfio/kamn/branches --paginate --jq 'length' | awk '{s+=$1} END{print s}'`
- `git fetch origin --prune`
- `git branch -r --merged origin/main`
- `git push origin --delete <merged-branch>`
- `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics
- Remote branch count reduced from 51 to `<=50`.
- Deleted branch lineage is merged-only and captured in evidence.
- Non-regression docs-contract tests remain green after lifecycle/spec updates.
