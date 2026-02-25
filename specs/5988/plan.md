# Plan: Issue #5988

## Approach
- Add `crates/kamn-core/tests/review_doc_helpers.rs` with canonical helpers:
  - `repo_root`
  - `parse_marker_value`
  - `parse_marker_text`
  - `parse_marker_usize`
  - `parse_marker_f64`
  - `parse_marker_csv`
- Update review contract files to import helper module via `#[path = "review_doc_helpers.rs"]`.
- Remove local duplicate helper implementations and route calls through shared helpers.
- Run focused governance contract tests for touched files.

## Affected Modules
- `crates/kamn-core/tests/review_doc_helpers.rs`
- `crates/kamn-core/tests/review_r48_spec_volume_guardrail_docs_contract.rs`
- `crates/kamn-core/tests/review_r49_docs_contract.rs`
- `crates/kamn-core/tests/review_r49_completed_milestone_closure_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_doc_contract_consolidation_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_governance_feature_rebalancing_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_governance_loop_mitigation_docs_contract.rs`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`

## Risks / Mitigations
- Risk: helper signature mismatch breaks existing tests.
  Mitigation: keep semantics consistent, use wrappers where callsites assume doc constant.
- Risk: overly broad refactor touches behavior.
  Mitigation: avoid assertion changes; only helper wiring.

## Interfaces / Contracts
- New internal test-only helper interface in `review_doc_helpers.rs`.
- No production/runtime interfaces changed.
