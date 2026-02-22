# Review Artifact Marker Contract

R43+ `gaps-and-issues` review artifacts must carry deterministic governance-vs-feature activity markers using key/value lines.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 43`:

- `governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1`
- `governance_activity_commit_count=<integer>`
- `feature_activity_commit_count=<integer>`
- `activity_total_commit_count=<integer>`
- `governance_activity_commit_ratio=<float>`
- `feature_activity_commit_ratio=<float>`

Contract invariants:

- `governance_activity_commit_count + feature_activity_commit_count = activity_total_commit_count`
- `governance_activity_commit_ratio ~= governance_activity_commit_count / activity_total_commit_count`
- `feature_activity_commit_ratio ~= feature_activity_commit_count / activity_total_commit_count`
- `governance_activity_commit_ratio + feature_activity_commit_ratio ~= 1.0`

This schema is enforced by `crates/kamn-core/tests/release_review_activity_ratio_docs_contract.rs`.

## Snapshot Semantics Contract (R50+)

R50+ review artifacts that include volatile ecosystem counts (for example remote branch heads) must
declare point-in-time snapshot semantics.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 50`:

- `review_snapshot_semantics_policy_schema_version=kamn.review.snapshot-semantics-policy.v1`
- `r<release>_review_snapshot_as_of_date=<YYYY-MM-DD>`
- `r<release>_review_branch_remote_head_count_contract_mode=informational_only`
- `r<release>_review_branch_reconciliation_issue_chain_count=<integer>`
- `r<release>_review_branch_reconciliation_issue_chain_max=1`

Optional informational marker:

- `r<release>_review_branch_remote_head_count_snapshot=<integer>`

Contract invariants:

- `r<release>_review_branch_remote_head_count_contract_mode` must remain `informational_only`
- `r<release>_review_branch_reconciliation_issue_chain_count <= r<release>_review_branch_reconciliation_issue_chain_max`
- `r<release>_review_branch_reconciliation_issue_chain_max = 1`

This schema is enforced by `crates/kamn-core/tests/review_r50_governance_loop_mitigation_docs_contract.rs`.

## Spec-Volume Non-Regression Ratchet Contract (R50+)

R50+ review artifacts must include a non-regression ratchet while spec-volume remediation remains active.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 50`:

- `r<release>_review_spec_volume_non_regression_schema_version=kamn.review.spec-volume-non-regression-ratchet.v1`
- `r<release>_review_spec_volume_non_regression_baseline_spec_dirs=<integer>`
- `r<release>_review_spec_volume_non_regression_baseline_module_count=<integer>`
- `r<release>_review_spec_volume_non_regression_ratio_max=<float>`
- `r<release>_review_spec_volume_non_regression_spec_dir_max=<integer>`

Contract invariants:

- current spec_dir_count <= non_regression_spec_dir_max
- current spec_to_module_ratio <= non_regression_ratio_max
- non_regression_baseline_spec_dirs <= non_regression_spec_dir_max

This schema is enforced by `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`.

## Doc-Contract Test-File Non-Regression Ratchet Contract (R50+)

R50+ review artifacts must include a non-regression ratchet for doc-contract test-file count while
doc-contract consolidation remediation is active.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 50`:

- `r<release>_review_doc_contract_non_regression_schema_version=kamn.review.doc-contract-non-regression-ratchet.v1`
- `r<release>_review_doc_contract_non_regression_baseline_test_file_count=<integer>`
- `r<release>_review_doc_contract_non_regression_max_test_file_count=<integer>`
- `r<release>_review_doc_contract_non_regression_count_formula=rg --files crates/kamn-core/tests | rg '_docs\\.rs$|docs_contract' | wc -l`

Contract invariants:

- current doc_contract_test_file_count <= non_regression_max
- non_regression_baseline_test_file_count <= non_regression_max_test_file_count

This schema is enforced by `crates/kamn-core/tests/review_r50_doc_contract_consolidation_docs_contract.rs`.
