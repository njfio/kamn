# Issue #5555 Plan - R50 Governance-Feature Activity Non-Regression Ratchet Enforcement

## Approach
1. Add RED assertions for governance-feature non-regression ratchet markers in `review_r50_governance_feature_rebalancing_docs_contract.rs`.
2. Add ratchet markers to `docs/review/gaps-and-issues-r50.md`.
3. Add ratchet schema/invariants to `docs/review/README.md`.
4. Re-run targeted review docs-contract lanes and quality gates.

## Affected Modules
- `crates/kamn-core/tests/review_r50_governance_feature_rebalancing_docs_contract.rs`
- `docs/review/gaps-and-issues-r50.md`
- `docs/review/README.md`

## Risks and Mitigations
- Risk: marker-name ambiguity with existing activity-ratio markers.
  - Mitigation: use dedicated `*_non_regression_*` marker keys.
- Risk: ratchet values drift from baseline markers.
  - Mitigation: integration assertions bind ratchet bounds to current R50 ratio markers.

## Interfaces / Contracts
- New markers:
  - `r50_review_governance_feature_non_regression_schema_version=kamn.review.governance-feature-non-regression-ratchet.v1`
  - `r50_review_governance_feature_non_regression_governance_ratio_max=<float>`
  - `r50_review_governance_feature_non_regression_feature_ratio_min=<float>`

## ADR
- Not required.
