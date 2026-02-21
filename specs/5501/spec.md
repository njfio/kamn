# Issue #5501 Spec - R49 Post-Publication Feature Provenance Markers

- Status: Accepted
- Issue: #5501
- Parent: #5469
- Milestone: R50.16 R49 production-feature provenance parity refresh

## Problem Statement
The R49 review artifact tracks post-publication branch/milestone/ignored-test revalidation but does not include deterministic provenance that production feature work resumed via `#5499` / `#5500`.

## Scope
In scope:
- Add deterministic feature provenance markers to `docs/review/gaps-and-issues-r49.md`.
- Update `crates/kamn-core/tests/review_r49_docs_contract.rs` to validate the new markers.
- Keep marker consistency checks deterministic.

Out of scope:
- New runtime feature implementation.
- Changes to R49 baseline historical snapshot markers.

## Acceptance Criteria
- AC-1: R49 review doc includes deterministic post-publication feature provenance markers for issue and PR.
- AC-2: R49 status highlights include explicit feature-delivery reconciliation statement.
- AC-3: `review_r49_docs_contract` validates the new markers and numeric consistency.
- AC-4: Targeted docs-contract tests pass.

## Conformance Cases
- C-01 (AC-1): marker `r49_review_post_publication_feature_issue=5499` exists.
- C-02 (AC-1): marker `r49_review_post_publication_feature_pr=5500` exists.
- C-03 (AC-2): status highlight explicitly references `#5499`/`#5500` production feature delivery.
- C-04 (AC-3): docs-contract test checks marker presence and parsed numeric equality.
- C-05 (AC-4): `cargo test -p kamn-core --test review_r49_docs_contract` passes.

## Success Metrics / Observable Signals
- R49 review artifact post-publication section now captures feature provenance parity.
- Docs-contract test enforces marker drift detection for feature provenance.
