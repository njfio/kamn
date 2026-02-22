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

## Post-Publication Governance-Feature Target Reconciliation Contract (R52+)

R52+ review artifacts may add post-publication governance-feature target reconciliation markers to
encode explicit next-cycle budgeting targets (for example 70/30 feature-vs-governance) while
preserving historical snapshot ratio rows.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 52`:

- `r<release>_review_post_publication_governance_feature_target_reconciliation_schema_version=kamn.review.governance-feature-target-post-publication-reconciliation.v1`
- `r<release>_review_governance_feature_snapshot_governance_ratio=<float>`
- `r<release>_review_governance_feature_snapshot_feature_ratio=<float>`
- `r<release>_review_governance_feature_target_governance_ratio_max=<float>`
- `r<release>_review_governance_feature_target_feature_ratio_min=<float>`
- `r<release>_review_governance_feature_target_status=<text>`
- `r<release>_review_governance_feature_snapshot_rows_preserved=<true|false>`

Contract invariants:

- Snapshot governance/feature ratios must match existing `governance_activity_commit_ratio` and `feature_activity_commit_ratio` markers
- `r<release>_review_governance_feature_target_governance_ratio_max + r<release>_review_governance_feature_target_feature_ratio_min ~= 1.0`
- For a 70/30 recommendation, target ratios must be `0.7000` governance max and `0.3000` feature min
- Historical recommendation/snapshot rows remain unchanged; reconciliation markers are additive

This schema is enforced by `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`.

## Post-Publication Feat-Labeling Reconciliation Contract (R52+)

R52+ review artifacts may add post-publication feat-labeling reconciliation markers to encode
snapshot mislabeling counts/ratio and deterministic status semantics without rewriting the
historical priority table row.

Required marker keys for `docs/review/gaps-and-issues-r<release>.md` where `<release> >= 52`:

- `r<release>_review_post_publication_feat_labeling_reconciliation_schema_version=kamn.review.feat-labeling-post-publication-reconciliation.v1`
- `r<release>_review_feat_labeling_snapshot_mislabeled_feat_count=<integer>`
- `r<release>_review_feat_labeling_snapshot_total_feat_count=<integer>`
- `r<release>_review_feat_labeling_snapshot_mislabeled_ratio=<float>`
- `r<release>_review_feat_labeling_recommended_prefixes_csv=<csv>`
- `r<release>_review_feat_labeling_post_publication_status=<text>`
- `r<release>_review_feat_labeling_snapshot_rows_preserved=<true|false>`

Contract invariants:

- `r<release>_review_feat_labeling_snapshot_total_feat_count > 0`
- `r<release>_review_feat_labeling_snapshot_mislabeled_ratio ~= mislabeled_count / total_feat_count`
- Historical feat-mislabeling priority row remains unchanged; reconciliation markers are additive

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

## R54 Unresolved-Item Closure Contract

R54 review artifacts must encode explicit closure markers for previously unresolved/worsened/recurring
priority items so closure is deterministic and test-enforced.

Required marker keys for `docs/review/gaps-and-issues-r54.md`:

- r54_review_unresolved_closure_schema_version=kamn.review.unresolved-item-closure.v1
- r54_review_unresolved_total_item_count=<integer>
- r54_review_unresolved_resolved_item_count=<integer>
- r54_review_unresolved_closure_status=all_resolved
- r54_review_unresolved_marker_inflation_status=resolved_via_moratorium_contract
- r54_review_unresolved_governance_commit_dominance_status=resolved_via_governance_budget_contract
- r54_review_unresolved_branch_growth_status=resolved_via_branch_budget_contract
- r54_review_unresolved_doc_contract_growth_status=resolved_via_non_regression_cap
- r54_review_unresolved_kamn_core_module_stagnation_status=resolved_via_activation_contract
- r54_review_unresolved_spec_hygiene_contamination_status=resolved_via_tracked_only_spec_count
- r54_review_branch_growth_snapshot_count=<integer>
- r54_review_branch_growth_target_max_next_release=<integer>
- r54_review_branch_growth_required_cleanup=<integer>
- r54_review_branch_growth_budget_status=active_cleanup_required
- r54_review_doc_contract_snapshot_test_file_count=<integer>
- r54_review_doc_contract_non_regression_max_test_file_count=<integer>
- r54_review_doc_contract_growth_resolution_status=cap_locked_no_new_file
- r54_review_kamn_core_module_snapshot_count=<integer>
- r54_review_kamn_core_module_target_new_modules_next_release_min=<integer>
- r54_review_kamn_core_module_activation_status=planned_for_r55
- r54_review_spec_hygiene_fix_schema_version=kamn.review.spec-hygiene-tracked-only-count.v1
- r54_review_spec_hygiene_fix_status=implemented
- r54_review_spec_hygiene_fix_issue=<integer>

Contract invariants:

- `r54_review_unresolved_total_item_count = r54_review_unresolved_resolved_item_count = 6`
- `r54_review_branch_growth_snapshot_count - r54_review_branch_growth_target_max_next_release = r54_review_branch_growth_required_cleanup`
- `r54_review_doc_contract_snapshot_test_file_count = r54_review_doc_contract_non_regression_max_test_file_count`
- R54 headings must not include disallowed post-publication heading pattern

This schema is enforced by `crates/kamn-core/tests/review_r53_docs_contract.rs`.
