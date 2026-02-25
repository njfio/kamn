#[path = "review_doc_helpers.rs"]
mod review_doc_helpers;

use review_doc_helpers::{parse_marker_f64, parse_marker_usize};

const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r50.md");
const REVIEW_MARKER_README: &str = include_str!("../../../docs/review/README.md");

#[test]
fn functional_r50_governance_loop_mitigation_markers_present() {
    assert!(REVIEW_MARKER_README.contains("review_snapshot_semantics_policy_schema_version"));
    assert!(REVIEW_MARKER_README.contains("kamn.review.snapshot-semantics-policy.v1"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_remote_head_count_contract_mode=informational_only"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_reconciliation_issue_chain_count=<integer>"));
    assert!(
        REVIEW_MARKER_README.contains("r<release>_review_branch_reconciliation_issue_chain_max=1")
    );

    assert!(DOC.contains(
        "review_snapshot_semantics_policy_schema_version=kamn.review.snapshot-semantics-policy.v1"
    ));
    assert!(DOC.contains("r50_review_snapshot_as_of_date=2026-02-21"));
    assert!(DOC.contains("r50_review_branch_remote_head_count_contract_mode=informational_only"));
    assert!(DOC.contains("r50_review_branch_reconciliation_issue_chain_count=0"));
    assert!(DOC.contains("r50_review_branch_reconciliation_issue_chain_max=1"));
    assert!(DOC.contains("r50_review_branch_remote_head_count_snapshot=51"));

    assert!(DOC.contains(
        "r50_review_governance_loop_mitigation_policy_schema_version=kamn.review.governance-loop-mitigation-policy.v1"
    ));
    assert!(DOC.contains("r50_review_marker_semantics=point_in_time_snapshot"));
    assert!(DOC.contains("r50_review_branch_count_marker_contract_mode=informational_only"));
    assert!(DOC.contains("r50_review_reconciliation_followup_issue_cap=1"));
    assert!(DOC.contains("r50_review_reconciliation_spec_artifact_cap=1"));
    assert!(DOC.contains("r50_review_reconciliation_baseline_issue_count=7"));
    assert!(DOC.contains("r50_review_reconciliation_baseline_commit_count=20"));
    assert!(DOC.contains("r50_review_reconciliation_baseline_spec_artifact_count=10"));
    assert!(DOC.contains("r50_review_reconciliation_expected_issue_reduction=6"));
    assert!(DOC.contains("r50_review_reconciliation_expected_spec_artifact_reduction=9"));
    assert!(DOC.contains("r50_review_spec_volume_baseline_spec_dirs=750"));
    assert!(DOC.contains("r50_review_spec_volume_baseline_module_count=92"));
    assert!(DOC.contains("r50_review_spec_volume_target_ratio_max=7.7"));
    assert!(DOC.contains("r50_review_spec_volume_target_spec_dir_max=708"));
    assert!(DOC.contains("r50_review_spec_volume_required_reduction=42"));
    assert!(DOC.contains("r50_review_spec_volume_remediation_status=active"));
}

#[test]
fn integration_r50_governance_loop_mitigation_marker_consistency() {
    let branch_reconciliation_chain_count =
        parse_marker_usize(DOC, "r50_review_branch_reconciliation_issue_chain_count");
    let branch_reconciliation_chain_max =
        parse_marker_usize(DOC, "r50_review_branch_reconciliation_issue_chain_max");
    let branch_count_snapshot =
        parse_marker_usize(DOC, "r50_review_branch_remote_head_count_snapshot");

    let baseline_issue_count =
        parse_marker_usize(DOC, "r50_review_reconciliation_baseline_issue_count");
    let issue_cap = parse_marker_usize(DOC, "r50_review_reconciliation_followup_issue_cap");
    let expected_issue_reduction =
        parse_marker_usize(DOC, "r50_review_reconciliation_expected_issue_reduction");

    let baseline_spec_artifact_count = parse_marker_usize(
        DOC,
        "r50_review_reconciliation_baseline_spec_artifact_count",
    );
    let spec_artifact_cap = parse_marker_usize(DOC, "r50_review_reconciliation_spec_artifact_cap");
    let expected_spec_artifact_reduction = parse_marker_usize(
        DOC,
        "r50_review_reconciliation_expected_spec_artifact_reduction",
    );

    let baseline_spec_dirs = parse_marker_usize(DOC, "r50_review_spec_volume_baseline_spec_dirs");
    let module_count = parse_marker_usize(DOC, "r50_review_spec_volume_baseline_module_count");
    let target_ratio_max = parse_marker_f64(DOC, "r50_review_spec_volume_target_ratio_max");
    let target_spec_dir_max = parse_marker_usize(DOC, "r50_review_spec_volume_target_spec_dir_max");
    let required_reduction = parse_marker_usize(DOC, "r50_review_spec_volume_required_reduction");

    assert_eq!(
        baseline_issue_count.saturating_sub(issue_cap),
        expected_issue_reduction
    );
    assert_eq!(
        baseline_spec_artifact_count.saturating_sub(spec_artifact_cap),
        expected_spec_artifact_reduction
    );

    let computed_target_spec_dir_max = (target_ratio_max * module_count as f64).floor() as usize;
    assert_eq!(computed_target_spec_dir_max, target_spec_dir_max);
    assert_eq!(
        baseline_spec_dirs.saturating_sub(target_spec_dir_max),
        required_reduction
    );
    assert_eq!(branch_reconciliation_chain_max, 1);
    assert!(branch_reconciliation_chain_count <= branch_reconciliation_chain_max);
    assert!(branch_count_snapshot > 0);
}
