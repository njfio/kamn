# Issue #5493 Spec - R49 Revalidation Provenance Reconciliation

- Status: Implemented
- Issue: #5493
- Parent: #3812
- Milestone: R50.12 R49 revalidation provenance reconciliation

## Problem Statement
R49 deterministic markers now report post-publication branch heads as `51`, but provenance currently references only issue `#5485` (the original revalidation snapshot), leaving branch-count reconciliation lineage implicit.

## Scope
In scope:
- Add explicit provenance statement for branch-count reconciliation via issue `#5491`.
- Add deterministic marker for reconciliation issue id.
- Preserve existing historical snapshot lineage to issue `#5485`.
- Update docs-contract tests to enforce both snapshot and reconciliation markers.

Out of scope:
- Runtime/product behavior changes.
- Any branch deletion workflow changes.

## Acceptance Criteria
- AC-1: `docs/review/gaps-and-issues-r49.md` includes explicit prose identifying branch-count reconciliation issue `#5491`.
- AC-2: Deterministic marker `r49_review_post_publication_branch_count_reconciliation_issue=5491` exists.
- AC-3: Existing marker `r49_review_post_publication_issue=5485` remains intact.
- AC-4: Docs-contract tests assert both provenance markers and pass.

## Conformance Cases
- C-01 (AC-1): revalidation section contains reconciliation provenance sentence referencing `#5491`.
- C-02 (AC-2): deterministic marker line exactly matches reconciliation marker/value.
- C-03 (AC-3): deterministic marker line for `r49_review_post_publication_issue=5485` remains present.
- C-04 (AC-4): `cargo test -p kamn-core --test review_r49_docs_contract` passes with new provenance assertions.

## Success Metrics / Observable Signals
- R49 review artifact provenance is explicit and internally consistent for the 51-head value.
- Docs-contract test suite remains green.
