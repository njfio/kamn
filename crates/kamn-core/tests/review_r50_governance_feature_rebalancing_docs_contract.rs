const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r50.md");
const REVIEW_MARKER_README: &str = include_str!("../../../docs/review/README.md");

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
fn functional_r50_governance_feature_rebalancing_markers_present() {
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_governance_feature_non_regression_schema_version=kamn.review.governance-feature-non-regression-ratchet.v1"
    ));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_governance_feature_non_regression_governance_ratio_max=<float>"
    ));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_governance_feature_non_regression_feature_ratio_min=<float>"));
    assert!(REVIEW_MARKER_README.contains(
        "current governance_activity_commit_ratio <= non_regression_governance_ratio_max"
    ));
    assert!(REVIEW_MARKER_README
        .contains("current feature_activity_commit_ratio >= non_regression_feature_ratio_min"));

    assert!(DOC.contains(
        "r50_review_governance_feature_rebalancing_schema_version=kamn.review.governance-feature-rebalancing-plan.v1"
    ));
    assert!(
        DOC.contains("r50_review_governance_feature_rebalancing_baseline_governance_commits=28")
    );
    assert!(DOC.contains("r50_review_governance_feature_rebalancing_baseline_feature_commits=3"));
    assert!(DOC.contains("r50_review_governance_feature_rebalancing_baseline_total_commits=31"));
    assert!(DOC.contains(
        "r50_review_governance_feature_rebalancing_target_feature_commit_ratio_min=0.25"
    ));
    assert!(DOC.contains(
        "r50_review_governance_feature_rebalancing_target_governance_commit_ratio_max=0.75"
    ));
    assert!(DOC.contains("r50_review_governance_feature_rebalancing_target_feature_commit_min=8"));
    assert!(
        DOC.contains("r50_review_governance_feature_rebalancing_required_feature_commit_delta=5")
    );
    assert!(DOC.contains(
        "r50_review_governance_feature_rebalancing_governance_commit_cap_for_ratio_target=23"
    ));
    assert!(DOC.contains("r50_review_governance_feature_rebalancing_issue_cap_per_release=3"));
    assert!(DOC.contains("r50_review_governance_feature_rebalancing_target_release=r53"));
    assert!(DOC.contains("r50_review_governance_feature_rebalancing_status=active"));
    assert!(DOC.contains(
        "r50_review_governance_feature_non_regression_schema_version=kamn.review.governance-feature-non-regression-ratchet.v1"
    ));
    assert!(
        DOC.contains("r50_review_governance_feature_non_regression_governance_ratio_max=0.9032")
    );
    assert!(DOC.contains("r50_review_governance_feature_non_regression_feature_ratio_min=0.0968"));
    assert!(DOC.contains(
        "Governance-feature rebalancing contract active (R50.20) targeting >=0.25 feature ratio (>=8 of 31 commits) by r53."
    ));
}

#[test]
fn integration_r50_governance_feature_rebalancing_markers_are_consistent() {
    let baseline_governance =
        parse_marker_usize("r50_review_governance_feature_rebalancing_baseline_governance_commits");
    let baseline_feature =
        parse_marker_usize("r50_review_governance_feature_rebalancing_baseline_feature_commits");
    let baseline_total =
        parse_marker_usize("r50_review_governance_feature_rebalancing_baseline_total_commits");

    let target_feature_ratio_min = parse_marker_f64(
        "r50_review_governance_feature_rebalancing_target_feature_commit_ratio_min",
    );
    let target_governance_ratio_max = parse_marker_f64(
        "r50_review_governance_feature_rebalancing_target_governance_commit_ratio_max",
    );
    let target_feature_commit_min =
        parse_marker_usize("r50_review_governance_feature_rebalancing_target_feature_commit_min");
    let required_feature_delta = parse_marker_usize(
        "r50_review_governance_feature_rebalancing_required_feature_commit_delta",
    );
    let governance_commit_cap = parse_marker_usize(
        "r50_review_governance_feature_rebalancing_governance_commit_cap_for_ratio_target",
    );
    let issue_cap_per_release =
        parse_marker_usize("r50_review_governance_feature_rebalancing_issue_cap_per_release");
    let current_governance_ratio = parse_marker_f64("governance_activity_commit_ratio");
    let current_feature_ratio = parse_marker_f64("feature_activity_commit_ratio");
    let non_regression_governance_ratio_max =
        parse_marker_f64("r50_review_governance_feature_non_regression_governance_ratio_max");
    let non_regression_feature_ratio_min =
        parse_marker_f64("r50_review_governance_feature_non_regression_feature_ratio_min");

    assert_eq!(baseline_governance + baseline_feature, baseline_total);
    assert!((target_feature_ratio_min + target_governance_ratio_max - 1.0).abs() <= 0.001);
    assert_eq!(
        target_feature_commit_min.saturating_sub(baseline_feature),
        required_feature_delta
    );
    assert_eq!(
        baseline_total.saturating_sub(target_feature_commit_min),
        governance_commit_cap
    );
    assert!(
        target_feature_commit_min as f64 / baseline_total as f64 >= target_feature_ratio_min,
        "feature commit target should satisfy minimum feature ratio"
    );
    assert!(
        governance_commit_cap as f64 / baseline_total as f64 <= target_governance_ratio_max + 0.001,
        "governance cap should satisfy maximum governance ratio"
    );
    assert!(
        issue_cap_per_release <= 3,
        "issue cap per release must stay bounded"
    );
    assert!(
        current_governance_ratio <= non_regression_governance_ratio_max + 0.0001,
        "current governance ratio must not exceed non-regression max"
    );
    assert!(
        current_feature_ratio + 0.0001 >= non_regression_feature_ratio_min,
        "current feature ratio must not drop below non-regression minimum"
    );
}
