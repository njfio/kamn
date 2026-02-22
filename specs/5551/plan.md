# Issue #5551 Plan - R50 Spec-Volume Non-Regression Ratchet Guardrail Enforcement

## Approach
1. Add RED assertions in existing `review_r50_spec_volume_remediation_docs_contract.rs` for non-regression ratchet markers and current-count enforcement.
2. Add ratchet markers to `docs/review/gaps-and-issues-r50.md`.
3. Add ratchet policy schema/invariants to `docs/review/README.md`.
4. Run targeted docs-contract and review marker validation lanes.

## Affected Modules
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `docs/review/gaps-and-issues-r50.md`
- `docs/review/README.md`

## Risks and Mitigations
- Risk: dynamic count logic drifts from established counting method.
  - Mitigation: use deterministic repository-local counting aligned with review evidence commands.
- Risk: ratchet too strict for legitimate structured remediation.
  - Mitigation: ratchet maxima set to current baseline-equivalent values and documented as explicit policy markers.

## Interfaces / Contracts
- New R50 ratchet markers:
  - `r50_review_spec_volume_non_regression_schema_version=kamn.review.spec-volume-non-regression-ratchet.v1`
  - `r50_review_spec_volume_non_regression_baseline_spec_dirs=<integer>`
  - `r50_review_spec_volume_non_regression_baseline_module_count=<integer>`
  - `r50_review_spec_volume_non_regression_ratio_max=<float>`
  - `r50_review_spec_volume_non_regression_spec_dir_max=<integer>`
- Invariants:
  - current spec-dir count `<= spec_dir_max`
  - current spec-to-module ratio `<= ratio_max`

## ADR
- Not required.
