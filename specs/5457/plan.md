# Issue #5457 Plan - Merged-Only Branch Cleanup

## Approach
1. Capture RED pre-cleanup evidence:
   - remote branch count
   - merged remote candidate list under `origin/codex/*`
2. Delete merged-only candidates via `git push origin --delete <branch>`.
3. Capture GREEN post-cleanup branch count.
4. Commit planning artifact with pre/post evidence and deletion inventory.
5. Run targeted docs regression suite.

## Affected Modules
- `docs/planning/2026-02-21-branch-hygiene-refresh-wave.md`
- `specs/5457/spec.md`
- `specs/5457/plan.md`
- `specs/5457/tasks.md`

## Risks / Mitigations
- Risk: deleting an unmerged branch.
  - Mitigation: candidate list must come strictly from `git branch -r --merged origin/main`.
- Risk: stale remote refs.
  - Mitigation: `git fetch origin --prune` before inventory.

## Interfaces / Contracts
- Branch-hygiene contract: merged-only deletions.
- Evidence contract: docs artifact includes exact command outputs and deleted names.

## Validation Strategy
- RED:
  - `git ls-remote --heads origin | wc -l`
  - `git branch -r --merged origin/main | rg '^origin/codex/'`
- GREEN:
  - rerun count command and verify reduction.
- REGRESSION:
  - `cargo test -p kamn-core --test docs_contract_wave3_harness -- --nocapture`
