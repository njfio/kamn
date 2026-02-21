# Issue #5495 Plan - Stale Branch Trim Execution

## Approach
1. Capture pre-cleanup count and enumerate `codex/issue-*` remote branches.
2. Deterministically filter candidates by closed-issue/no-open-PR/non-merged criteria.
3. Select one candidate branch and delete it.
4. Capture post-cleanup count and evidence.

## Affected Modules
- `specs/milestones/r50-13-branch-hygiene-stale-closed-issue-branch-trim/index.md`
- `specs/5495/{spec,plan,tasks,evidence}.md`

## Risks / Mitigations
- Risk: deleting still-needed branch.
  - Mitigation: enforce closed issue + no open PR + non-merged criteria and bounded single deletion.

## Interfaces / Contracts
- Branch hygiene governance process only.

## Validation Strategy
- `git ls-remote --heads origin | wc -l`
- `gh issue view <id> --json state`
- `gh pr list --state open --json headRefName`
- `git merge-base --is-ancestor <sha> origin/main`
- `gh api -X DELETE repos/njfio/kamn/git/refs/heads/<branch>`
