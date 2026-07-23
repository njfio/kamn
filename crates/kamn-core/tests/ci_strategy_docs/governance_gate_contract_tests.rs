use super::fairness_deletion_support::assert_contains_all;
use super::DOC;

#[test]
fn doc_contains_governance_feature_commit_ratio_gate_markers() {
    assert_governance_gate_commands();
    assert_governance_gate_schema_markers();
    assert_governance_gate_policy_markers();
}

#[test]
fn doc_contains_review_document_freeze_gate_markers() {
    assert_contains_all(
        DOC,
        &[
            "## Review-Document Freeze Fast Gate",
            "python3 scripts/ci/check_review_document_freeze.py --changed-files-file /tmp/pr-changed-files.txt --freeze-manifest docs/review/review-document-freeze.manifest --output-json /tmp/review-document-freeze-report.json",
            "bash scripts/ci/test_check_review_document_freeze.sh",
            "git diff --name-only <base_sha>..<head_sha>",
            "ci-review-document-freeze.json",
            "review_document_freeze_schema_version=kamn.ci.review-document-freeze-gate-report.v1",
            "review_document_freeze_reason_taxonomy_version=kamn.ci.review-document-freeze-gate-reason-taxonomy.v1",
            "review_document_freeze_reason_codes_csv=review_document_freeze_changed_files_missing,review_document_freeze_manifest_missing,review_document_freeze_manifest_invalid,review_document_freeze_violation_detected",
            "review_document_freeze_manifest_path=docs/review/review-document-freeze.manifest",
            "review_document_freeze_scope=docs/review/gaps-and-issues-r*.md",
        ],
        "review document freeze",
    );
}

fn assert_governance_gate_commands() {
    assert_contains_all(
        DOC,
        &[
            "## Governance/Feature Commit-Ratio Fast Gate",
            "python3 scripts/ci/check_governance_feature_commit_ratio.py --commit-subjects-file /tmp/pr-commit-subjects.txt --window-size 50 --max-governance-ratio 0.20 --output-json /tmp/governance-feature-commit-ratio-report.json",
            "bash scripts/ci/test_check_governance_feature_commit_ratio.sh",
            "source .ci/governance-feature-commit-ratio-moratorium.env",
            "git log --no-merges --pretty=format:%s \"${PR_BASE_SHA}..${PR_HEAD_SHA}\"",
            "ci-governance-feature-commit-ratio.json",
        ],
        "governance feature-commit ratio command",
    );
}

fn assert_governance_gate_schema_markers() {
    assert_contains_all(
        DOC,
        &[
            "governance_feature_commit_ratio_schema_version=kamn.ci.governance-feature-commit-ratio-report.v1",
            "governance_feature_commit_ratio_reason_taxonomy_version=kamn.ci.governance-feature-commit-ratio-reason-taxonomy.v1",
            "governance_feature_commit_ratio_reason_codes_csv=governance_commit_subjects_empty,governance_commit_subject_unclassified,governance_commit_ratio_threshold_exceeded",
            "governance_feature_commit_ratio_threshold_max=0.20",
            "governance_feature_commit_ratio_feature_ratio_min=0.80",
            "governance_feature_commit_ratio_window_size=50",
            "governance_feature_commit_ratio_scope=pull_request_non_merge_commits",
            "governance_feature_commit_ratio_policy_source=base_branch",
            "governance_feature_commit_ratio_classification_mode=changed_path_surface",
        ],
        "governance feature-commit ratio schema",
    );
}

fn assert_governance_gate_policy_markers() {
    assert_contains_all(
        DOC,
        &[
            "governance_feature_commit_ratio_activation_base_sha_file=.ci/governance-feature-commit-ratio-moratorium.env",
            "governance_feature_commit_ratio_activation_base_sha=d2c2fe1b901a1d53ea419f31778e1d836f2b1323",
            "governance_feature_commit_ratio_activation_scope=post_moratorium_commits_only",
            "governance_feature_commit_ratio_activation_base_status=head_at_activation_base",
            "governance_feature_commit_ratio_preactivation_rerun_status=head_precedes_activation_base",
            "governance_feature_commit_ratio_non_merge_only=true",
        ],
        "governance feature-commit ratio policy",
    );
}
