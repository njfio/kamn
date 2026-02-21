# Issue #5493 Plan - R49 Provenance Marker Reconciliation

## Approach
1. Add a post-reconciliation provenance sentence in `docs/review/gaps-and-issues-r49.md`.
2. Add deterministic marker `r49_review_post_publication_branch_count_reconciliation_issue=5491` while retaining existing `r49_review_post_publication_issue=5485` marker.
3. Update docs-contract tests to assert the new marker and provenance sentence.
4. Verify via targeted docs-contract test and format check.

## Affected Modules
- `docs/review/gaps-and-issues-r49.md`
- `crates/kamn-core/tests/review_r49_docs_contract.rs`
- `specs/milestones/r50-12-r49-revalidation-provenance-reconciliation/index.md`
- `specs/5493/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: unintentionally replacing historical marker `#5485`.
  - Mitigation: assert both markers in tests.

## Interfaces / Contracts
- Documentation contract only.

## Validation Strategy
- `cargo test -p kamn-core --test review_r49_docs_contract`
- `cargo fmt --check`
