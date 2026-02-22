# Issue #5553 Plan - R50 Doc-Contract Test-File Non-Regression Ratchet Enforcement

## Approach
1. Add RED assertions in `review_r50_doc_contract_consolidation_docs_contract.rs` for required ratchet markers and dynamic non-regression checks.
2. Add ratchet markers to `docs/review/gaps-and-issues-r50.md`.
3. Add ratchet marker schema and invariants to `docs/review/README.md`.
4. Re-run targeted review docs-contract lanes and quality gates.

## Affected Modules
- `crates/kamn-core/tests/review_r50_doc_contract_consolidation_docs_contract.rs`
- `docs/review/gaps-and-issues-r50.md`
- `docs/review/README.md`

## Risks and Mitigations
- Risk: inconsistent counting method across docs and tests.
  - Mitigation: declare and enforce one deterministic formula marker.
- Risk: ratchet too strict for ongoing refactors.
  - Mitigation: initialize max to current baseline while remediation remains active.

## Interfaces / Contracts
- New R50 markers:
  - `r50_review_doc_contract_non_regression_schema_version=kamn.review.doc-contract-non-regression-ratchet.v1`
  - `r50_review_doc_contract_non_regression_baseline_test_file_count=<integer>`
  - `r50_review_doc_contract_non_regression_max_test_file_count=<integer>`
  - `r50_review_doc_contract_non_regression_count_formula=rg --files crates/kamn-core/tests | rg '_docs\\.rs$|docs_contract' | wc -l`

## ADR
- Not required.
