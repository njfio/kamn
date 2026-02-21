const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r50.md");

fn parse_marker_usize(marker_key: &str) -> usize {
    let needle = format!("{marker_key}=");
    let line = DOC
        .lines()
        .find(|line| line.contains(needle.as_str()))
        .unwrap_or_else(|| panic!("missing marker {marker_key}"));
    let value = line
        .split_once(needle.as_str())
        .unwrap_or_else(|| panic!("marker {marker_key} missing '=' separator"))
        .1
        .trim_matches('`')
        .trim();
    value
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("marker {marker_key} should be an unsigned integer: {value}"))
}

fn parse_marker_f64(marker_key: &str) -> f64 {
    let needle = format!("{marker_key}=");
    let line = DOC
        .lines()
        .find(|line| line.contains(needle.as_str()))
        .unwrap_or_else(|| panic!("missing marker {marker_key}"));
    let value = line
        .split_once(needle.as_str())
        .unwrap_or_else(|| panic!("marker {marker_key} missing '=' separator"))
        .1
        .trim_matches('`')
        .trim();
    value
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("marker {marker_key} should be a number: {value}"))
}

#[test]
fn functional_r50_governance_loop_mitigation_markers_present() {
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
    let baseline_issue_count = parse_marker_usize("r50_review_reconciliation_baseline_issue_count");
    let issue_cap = parse_marker_usize("r50_review_reconciliation_followup_issue_cap");
    let expected_issue_reduction =
        parse_marker_usize("r50_review_reconciliation_expected_issue_reduction");

    let baseline_spec_artifact_count =
        parse_marker_usize("r50_review_reconciliation_baseline_spec_artifact_count");
    let spec_artifact_cap = parse_marker_usize("r50_review_reconciliation_spec_artifact_cap");
    let expected_spec_artifact_reduction =
        parse_marker_usize("r50_review_reconciliation_expected_spec_artifact_reduction");

    let baseline_spec_dirs = parse_marker_usize("r50_review_spec_volume_baseline_spec_dirs");
    let module_count = parse_marker_usize("r50_review_spec_volume_baseline_module_count");
    let target_ratio_max = parse_marker_f64("r50_review_spec_volume_target_ratio_max");
    let target_spec_dir_max = parse_marker_usize("r50_review_spec_volume_target_spec_dir_max");
    let required_reduction = parse_marker_usize("r50_review_spec_volume_required_reduction");

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
}
