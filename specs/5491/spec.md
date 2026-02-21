# Issue #5491 Spec - R49 Branch-Count Marker Reconciliation

- Status: Implemented
- Issue: #5491
- Parent: #3812
- Milestone: R50.11 R49 post-publication branch-count reconciliation

## Problem Statement
The R49 review artifact still reports post-publication remote branch heads as `52`, while current verified repository state is `51`.

## Scope
In scope:
- Reconcile post-publication branch-count values in `docs/review/gaps-and-issues-r49.md` from `52` to `51`.
- Keep status-highlight prose and deterministic markers internally consistent.
- Update docs-contract tests to assert the reconciled value.

Out of scope:
- Runtime/product behavior changes.
- Any unmerged branch deletion workflows.

## Acceptance Criteria
- AC-1: R49 post-publication branch-count snapshot/output sections use `51` consistently.
- AC-2: R49 deterministic marker `r49_review_post_publication_branch_remote_head_count` equals `51`.
- AC-3: Docs-contract tests assert reconciled values and pass.

## Conformance Cases
- C-01 (AC-1): `docs/review/gaps-and-issues-r49.md` lines in revalidation snapshot/output/highlights show `51`.
- C-02 (AC-2): deterministic marker line ends with `=51`.
- C-03 (AC-3): `cargo test -p kamn-core --test review_r49_docs_contract` passes with `51` assertions.

## Success Metrics / Observable Signals
- No remaining post-publication `52` branch-count markers in R49 review artifact.
- Docs-contract suite remains green.
