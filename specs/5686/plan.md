# Plan: #5686 Prune Merged Codex Issue Branches

## Approach
1. Capture baseline remote branch count from `origin`.
2. Enumerate open PR head branches to protect from deletion.
3. Build merged candidate set:
   - from `git branch -r --merged origin/main`
   - include only `origin/codex/issue-*`
   - exclude protected open-PR heads
4. Delete each candidate via `git push origin --delete <branch>`.
5. Capture post-cleanup remote branch count and summarize deltas.

## Affected Modules
- `specs/5686/spec.md`
- `specs/5686/plan.md`
- `specs/5686/tasks.md`

## Risks and Mitigations
- Risk: deleting a branch that still has active work.
- Mitigation: restrict deletion to branches merged into `origin/main` and not used by open PRs.

- Risk: transient remote failures during delete.
- Mitigation: run deletes per-branch and retain command output for traceability.

## Interfaces / Contracts
- Candidate filter contract:
  - Input: remote branch inventory + `origin/main` merge state + open PR head list.
  - Output: deletable branch list matching `codex/issue-*`.

## ADR
- Not required. No dependency/protocol/API/architecture change.
