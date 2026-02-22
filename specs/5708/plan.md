# Plan: #5708 Reconcile R52 Branch Hygiene Drift with Merged-Only Cleanup and Docs-Contract Markers

## Approach
1. Capture pre-cleanup remote branch count and merged-only candidate set.
2. Execute bounded remote deletions for merged-only candidates.
3. Capture post-cleanup count and update R52 review doc with reconciliation markers.
4. Add `kamn-core` docs-contract test for marker presence/arithmetic consistency.
5. Run targeted conformance checks (`cargo test` for new contract, fmt, clippy).

## Affected Modules
- `docs/review/gaps-and-issues-r52.md`
- `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs` (new)
- `specs/5708/{spec.md,plan.md,tasks.md}`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: accidental deletion of unmerged branches.
  Mitigation: compute candidates from `git branch -r --merged origin/main` and delete only those candidates.

- Risk: branch inventory changes concurrently during cleanup.
  Mitigation: capture deterministic pre/post count markers and document command evidence used.

- Risk: historical baseline confusion in R52 review text.
  Mitigation: keep original baseline section unchanged; append explicit reconciliation subsection with timestamped markers.

## Interfaces / Contracts
- R52 reconciliation marker schema in `docs/review/gaps-and-issues-r52.md`:
  - `r52_review_post_publication_branch_cleanup_schema_version=kamn.review.branch-hygiene-post-publication-cleanup.v1`
  - `r52_review_branch_remote_head_count_pre_cleanup=<integer>`
  - `r52_review_branch_remote_head_count_deleted=<integer>`
  - `r52_review_branch_remote_head_count_post_cleanup=<integer>`
- Marker arithmetic contract: `pre - deleted = post` and `post <= pre`.

## ADR
- Not required: governance process/evidence update only.
