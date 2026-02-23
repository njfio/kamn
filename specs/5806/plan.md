# Plan: Issue #5806 - Branch Budget Cleanup Tranche

- Issue: #5806
- Status: Completed
- Spec: `specs/5806/spec.md`

## Approach
1. Collect pre-cleanup branch inventory and merged-lineage list against `origin/main`.
2. Reconcile merged-lineage branch heads by pruning stale remote-tracking refs and verifying merged candidates exclude `origin/main` and `origin/HEAD`.
3. Validate post-cleanup remote count (`<=50`).
4. Update milestone index and finalize lifecycle/process markers.

## Affected Artifacts
- `specs/5806/spec.md`
- `specs/5806/plan.md`
- `specs/5806/tasks.md`
- `specs/milestones/r55-e2e-evidence-step-inventory-parity/index.md`

## Risks and Mitigations
- Risk: accidental deletion of active branch.
  - Mitigation: restrict selection to `git branch -r --merged origin/main` and explicitly exclude `origin/main` and `origin/HEAD`.
- Risk: nondeterministic evidence.
  - Mitigation: capture pre/post counts and deleted branch names in issue/PR comments.

## Verification Strategy
- Verify pre-count is 52.
- Verify exactly two `git push origin --delete ...` operations succeed.
- Verify post-count is 50.
- Run `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture` as regression sanity for R54+ docs contract lane.
