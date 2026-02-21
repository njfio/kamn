# Issue #5497 Spec - R49 Branch-Count Marker Reconciliation (51 -> 50)

- Status: Implemented
- Issue: #5497
- Parent: #3812
- Milestone: R50.14 R49 branch-count marker reconciliation after stale trim

## Problem Statement
R49 post-publication markers currently report branch count `51` with reconciliation marker `5491`, but repository state after `#5495` is `50`.

## Scope
In scope:
- Update R49 post-publication branch-count markers/highlights from `51` to `50`.
- Update branch-count reconciliation provenance marker from `5491` to `5495`.
- Update docs-contract assertions to enforce reconciled values.

Out of scope:
- Runtime/product behavior changes.

## Acceptance Criteria
- AC-1: R49 post-publication branch-count values are `50` consistently.
- AC-2: `r49_review_post_publication_branch_count_reconciliation_issue=5495` marker is present.
- AC-3: Docs-contract test asserts updated values and passes.

## Conformance Cases
- C-01 (AC-1): snapshot/output/highlight/marker lines show `50`.
- C-02 (AC-2): reconciliation marker value equals `5495`.
- C-03 (AC-3): `cargo test -p kamn-core --test review_r49_docs_contract` passes.

## Success Metrics / Observable Signals
- No stale `51` branch-count references remain in R49 post-publication sections.
- Docs-contract suite stays green.
