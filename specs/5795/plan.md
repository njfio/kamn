# Plan: Issue #5795 — Execute Merged-Only Remote Branch Cleanup to <=50 Heads

- Issue: #5795
- Spec: `specs/5795/spec.md`
- Status: Reviewed
- Last Updated: 2026-02-22

## Implementation Approach
1. Capture baseline remote head count.
2. Fetch and compute open-PR head refs (safety exclusions).
3. Enumerate remote branches and identify cleanup candidates via:
   - direct merge ancestry (`git merge-base --is-ancestor origin/<branch> origin/main`), or
   - merged PR lineage (`gh pr list --state merged --head <branch>`).
4. Exclude protected names (`main`, `origin`) and open-PR heads.
5. Delete merged candidates in one deterministic batch sufficient to reach <=50.
6. Recount heads and verify target.
7. Record evidence in issue comments and finalize lifecycle/milestone artifacts.

## Affected Modules
- `specs/5795/{spec.md,plan.md,tasks.md}`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- GitHub remote branch namespace (operational)

## Risks / Mitigations
- Risk: deleting active branch.
  - Mitigation: merged-PR/ancestor lineage check + open-PR exclusion + protected-name exclusion.
- Risk: insufficient deletions to reach target.
  - Mitigation: compute required deletion count before executing and re-run count.

## Interfaces / Contracts
- Git remote branch refs under `origin/*`.
- Merge safety contract: deleted ref must have merged lineage (direct ancestor of `origin/main` or merged PR evidence).

## ADR
- None required.
