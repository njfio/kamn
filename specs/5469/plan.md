# Issue #5469 Plan - R49 Review Artifact Publication

## Approach
1. RED: add docs-contract test expecting `gaps-and-issues-r49.md` marker set before file exists.
2. Capture baseline evidence commands for branch heads, open milestones/issues, and ignored-test drift status.
3. Publish `docs/review/gaps-and-issues-r49.md` with deterministic markers and concise status table.
4. GREEN: rerun docs-contract test and format checks.

## Affected Modules
- `docs/review/gaps-and-issues-r49.md` (new)
- `crates/kamn-core/tests/review_r49_docs_contract.rs` (new)
- `specs/milestones/r49-3-review-artifact-publication-baseline-refresh/index.md`
- `specs/5469/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: baseline marker drift if capture commands run at different times.
  - Mitigation: capture command outputs once and pin marker values in artifact.
- Risk: overfitting docs-contract to narrative content.
  - Mitigation: assert only deterministic marker keys and numeric consistency.

## Interfaces / Contracts
- `r49_review_artifact_schema_version=kamn.review.gaps-and-issues-r49.v1`

## Validation Strategy
- RED:
  - `cargo test -p kamn-core --test review_r49_docs_contract -- --nocapture`
- GREEN/REGRESSION:
  - `cargo test -p kamn-core --test review_r49_docs_contract -- --nocapture`
  - `cargo fmt --check`
