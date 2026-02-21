# Issue #5424 Plan — Deterministic Merged-Only Branch Cleanup

## Approach
1. Add a docs contract test that enforces refreshed branch-hygiene markers in `docs/review/gaps-and-issues-r45.md`.
2. Capture pre-cleanup branch count and merged-only candidate list.
3. Delete merged remote branches in a bounded batch.
4. Capture post-cleanup branch count and update docs markers.
5. Run targeted docs contract test + fmt/clippy gates.

## Affected Modules
- `docs/review/gaps-and-issues-r45.md`
- `crates/kamn-core/tests/review_branch_hygiene_docs_contract.rs` (new)
- `specs/5424/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: deleting active branch accidentally.
  - Mitigation: delete only branches listed by merged-to-main query and exclude `main`/`HEAD`.
- Risk: branch count drifts during execution from concurrent branch creation.
  - Mitigation: use immediate post-cleanup measurement and document command timestamped in issue/PR.

## Interfaces / Contracts
- Branch cleanup contract: `git branch -r --merged origin/main` filtered to remove `HEAD` and `main`.
- Review docs contract markers for branch baseline + evidence command stay synchronized.

## Validation Strategy
- Red: docs contract test fails before marker updates.
- Green: docs update + branch cleanup + docs contract test pass.
- Verify: targeted test + fmt + clippy + issue process log updates.
