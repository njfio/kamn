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

## Post-Publication Branch Cleanup Reconciliation Contract (R52+)

R52+ review artifacts may include post-publication branch hygiene reconciliation waves to document
merged-only cleanup attempts without rewriting point-in-time review snapshots.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 52`:

- `r<release>_review_post_publication_branch_cleanup_schema_version=kamn.review.branch-hygiene-post-publication-cleanup.v1`
- `r<release>_review_branch_remote_head_count_pre_cleanup=<integer>`
- `r<release>_review_branch_remote_head_count_deleted=<integer>`
- `r<release>_review_branch_remote_head_count_post_cleanup=<integer>`

Optional informational marker:

- `r<release>_review_branch_remote_head_count_baseline_snapshot=<integer>`

Contract invariants:

- `r<release>_review_branch_remote_head_count_pre_cleanup - r<release>_review_branch_remote_head_count_deleted = r<release>_review_branch_remote_head_count_post_cleanup`
- `r<release>_review_branch_remote_head_count_post_cleanup <= r<release>_review_branch_remote_head_count_pre_cleanup`

This schema is enforced by `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`.

## Post-Publication Quality-Gate Reconciliation Contract (R52+)

R52+ review artifacts may add post-publication quality-gate reconciliation markers to document
that critical as-of snapshot regressions were fixed later, without rewriting snapshot baselines.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 52`:

- `r<release>_review_post_publication_quality_gate_reconciliation_schema_version=kamn.review.quality-gate-post-publication-reconciliation.v1`
- `r<release>_review_workspace_quality_gate_status_post_publication=<pass|fail>`
- `r<release>_review_cli_compile_status_post_publication=<resolved|unresolved>`
- `r<release>_review_activity_ratio_marker_parse_status_post_publication=<resolved|unresolved>`
- `r<release>_review_workspace_quality_gate_command_post_publication=cargo test --workspace --locked --all-features --no-fail-fast`
- `r<release>_review_activity_ratio_marker_parse_command_post_publication=cargo test -p kamn-core --test release_review_activity_ratio_docs_contract`

Contract invariants:

- `r<release>_review_workspace_quality_gate_status_post_publication` must be either `pass` or `fail`
- `r<release>_review_cli_compile_status_post_publication` must be either `resolved` or `unresolved`
- `r<release>_review_activity_ratio_marker_parse_status_post_publication` must be either `resolved` or `unresolved`
- Snapshot header and baseline lines remain historical and are not rewritten by post-publication markers

This schema is enforced by `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`.

## Post-Publication Code-Quality Status Reconciliation Contract (R52+)

R52+ review artifacts may add post-publication code-quality status reconciliation markers to record
resolved quality-gate posture while preserving historical baseline wording in Section 4.2.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 52`:

- `r<release>_review_post_publication_code_quality_status_reconciliation_schema_version=kamn.review.code-quality-status-post-publication-reconciliation.v1`
- `r<release>_review_code_quality_snapshot_status=<text>`
- `r<release>_review_code_quality_post_publication_workspace_gate_status=<pass|fail>`
- `r<release>_review_code_quality_post_publication_cli_compile_status=<resolved|unresolved>`
- `r<release>_review_code_quality_snapshot_rows_preserved=<true|false>`

Contract invariants:

- Workspace/CLI post-publication code-quality statuses must align with post-publication quality-gate reconciliation markers
- Baseline Section 4.2 text remains unchanged; reconciliation markers are additive

This schema is enforced by `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`.

## Post-Publication Branch-Hygiene Status Reconciliation Contract (R52+)

R52+ review artifacts may add post-publication branch-hygiene status reconciliation markers to
capture current posture while preserving historical snapshot wording in headings and priority rows.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 52`:

- `r<release>_review_post_publication_branch_hygiene_status_reconciliation_schema_version=kamn.review.branch-hygiene-status-post-publication-reconciliation.v1`
- `r<release>_review_branch_hygiene_snapshot_status=<text>`
- `r<release>_review_branch_hygiene_snapshot_branch_count=<integer>`
- `r<release>_review_branch_hygiene_post_publication_pre_cleanup_count=<integer>`
- `r<release>_review_branch_hygiene_post_publication_post_cleanup_count=<integer>`
- `r<release>_review_branch_hygiene_post_publication_status=<text>`
- `r<release>_review_branch_hygiene_snapshot_rows_preserved=<true|false>`

Contract invariants:

- Status reconciliation snapshot/pre/post counts must align with branch cleanup reconciliation markers
- `r<release>_review_branch_hygiene_post_publication_post_cleanup_count <= r<release>_review_branch_hygiene_snapshot_branch_count`
- Baseline heading/priority row text remains unchanged; reconciliation markers are additive

This schema is enforced by `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`.

## Post-Publication Priority Summary Reconciliation Contract (R52+)

R52+ review artifacts may add post-publication priority-summary reconciliation markers to report
resolved status for historical priority rows while preserving baseline snapshot table text.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 52`:

- `r<release>_review_post_publication_priority_reconciliation_schema_version=kamn.review.priority-summary-post-publication-reconciliation.v1`
- `r<release>_review_priority_critical_cli_compile_status_post_publication=<resolved|unresolved>`
- `r<release>_review_priority_medium_activity_ratio_marker_status_post_publication=<resolved|unresolved>`
- `r<release>_review_priority_high_spec_volume_guardrail_status_post_publication=<within_guardrail|breached>`
- `r<release>_review_priority_summary_snapshot_preserved=<true|false>`

Contract invariants:

- Priority critical CLI status must align with `r<release>_review_cli_compile_status_post_publication`
- Priority medium activity-ratio marker status must align with `r<release>_review_activity_ratio_marker_parse_status_post_publication`
- Priority high spec-volume guardrail status must align with `r<release>_review_spec_volume_guardrail_post_publication_status`
- Historical priority table rows remain unchanged; reconciliation markers are additive

This schema is enforced by `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`.

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

## Post-Publication Spec-Volume Reduction Tranche Contract (R52+)

R52+ review artifacts may publish post-publication remediation tranche markers documenting
deterministic top-level `specs/` directory-count reductions without rewriting as-of snapshot baselines.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 52`:

- `r<release>_review_post_publication_spec_volume_reduction_schema_version=kamn.review.spec-volume-post-publication-reduction.v1`
- `r<release>_review_spec_volume_reduction_tranche_pre_count=<integer>`
- `r<release>_review_spec_volume_reduction_tranche_deleted_count=<integer>`
- `r<release>_review_spec_volume_reduction_tranche_post_count=<integer>`

Optional evidence markers:

- `r<release>_review_spec_volume_reduction_evidence_command_pre=<command>`
- `r<release>_review_spec_volume_reduction_evidence_command_post=<command>`

Contract invariants:

- `r<release>_review_spec_volume_reduction_tranche_pre_count - r<release>_review_spec_volume_reduction_tranche_deleted_count = r<release>_review_spec_volume_reduction_tranche_post_count`
- `r<release>_review_spec_volume_reduction_tranche_post_count <= r<release>_review_spec_volume_reduction_tranche_pre_count`

This schema is enforced by `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`.

## Post-Publication Spec-Volume Guardrail Reconciliation Contract (R52+)

R52+ review artifacts may add post-publication guardrail reconciliation markers to document that
spec-volume remediation moved current state back within guardrail bounds, without rewriting the
historical as-of snapshot baseline.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 52`:

- `r<release>_review_post_publication_spec_volume_guardrail_reconciliation_schema_version=kamn.review.spec-volume-guardrail-post-publication-reconciliation.v1`
- `r<release>_review_spec_volume_guardrail_snapshot_spec_dir_count=<integer>`
- `r<release>_review_spec_volume_guardrail_snapshot_module_count=<integer>`
- `r<release>_review_spec_volume_guardrail_post_publication_spec_dir_count=<integer>`
- `r<release>_review_spec_volume_guardrail_post_publication_module_count=<integer>`
- `r<release>_review_spec_volume_guardrail_post_publication_ratio=<float>`
- `r<release>_review_spec_volume_guardrail_target_ratio_max=<float>`
- `r<release>_review_spec_volume_guardrail_post_publication_status=<within_guardrail|breached>`

Contract invariants:

- `r<release>_review_spec_volume_guardrail_post_publication_spec_dir_count <= r<release>_review_spec_volume_guardrail_snapshot_spec_dir_count`
- `r<release>_review_spec_volume_guardrail_post_publication_ratio <= r<release>_review_spec_volume_guardrail_target_ratio_max` when status is `within_guardrail`
- Historical snapshot markers remain present and unchanged

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

## Governance-Feature Activity Non-Regression Ratchet Contract (R50+)

R50+ review artifacts must include governance-feature activity ratio non-regression bounds while
rebalancing remediation is active.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 50`:

- `r<release>_review_governance_feature_non_regression_schema_version=kamn.review.governance-feature-non-regression-ratchet.v1`
- `r<release>_review_governance_feature_non_regression_governance_ratio_max=<float>`
- `r<release>_review_governance_feature_non_regression_feature_ratio_min=<float>`

Contract invariants:

- current governance_activity_commit_ratio <= non_regression_governance_ratio_max
- current feature_activity_commit_ratio >= non_regression_feature_ratio_min

This schema is enforced by `crates/kamn-core/tests/review_r50_governance_feature_rebalancing_docs_contract.rs`.
