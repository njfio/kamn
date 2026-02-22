const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r49.md");

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

#[test]
fn functional_r49_review_markers_present() {
    assert!(DOC.contains("r49_review_artifact_schema_version=kamn.review.gaps-and-issues-r49.v1"));
    assert!(DOC.contains("r49_review_baseline_capture_date=2026-02-21"));
    assert!(DOC.contains("r49_review_baseline_open_issue_count=1"));
    assert!(DOC.contains("r49_review_baseline_open_milestone_count=1"));
    assert!(DOC.contains("r49_review_ignored_test_inventory_count=12"));
    assert!(DOC.contains("r49_review_ignored_test_periodic_review_status=completed"));
    assert!(DOC.contains("r49_review_milestone_closure_wave_closed_ids_csv=94,95,96,97"));
    assert!(DOC.contains("r49_review_post_publication_revalidation_date=2026-02-21"));
    assert!(DOC.contains("r49_review_post_publication_issue=5485"));
    assert!(DOC.contains("r49_review_post_publication_feature_issue=5499"));
    assert!(DOC.contains("r49_review_post_publication_feature_pr=5500"));
    assert!(DOC.contains("r49_review_post_publication_open_issue_count=0"));
    assert!(DOC.contains("r49_review_post_publication_open_milestone_count=0"));
    assert!(DOC.contains("r49_review_post_publication_ignored_test_inventory_count=12"));
    assert!(DOC.contains(
        "Publication snapshot values above remain historical to R49.3 artifact publication time."
    ));
    assert!(
        DOC.contains(
            "Post-publication production feature delivery is reconciled via issue `#5499` and PR `#5500`."
        )
    );
}

#[test]
fn integration_r49_review_marker_consistency() {
    let baseline_open_issue_count = parse_marker_usize("r49_review_baseline_open_issue_count");
    let baseline_open_milestone_count =
        parse_marker_usize("r49_review_baseline_open_milestone_count");
    let baseline_ignored_count = parse_marker_usize("r49_review_ignored_test_inventory_count");
    let post_open_issue_count = parse_marker_usize("r49_review_post_publication_open_issue_count");
    let post_open_milestone_count =
        parse_marker_usize("r49_review_post_publication_open_milestone_count");
    let post_ignored_count =
        parse_marker_usize("r49_review_post_publication_ignored_test_inventory_count");
    let post_feature_issue = parse_marker_usize("r49_review_post_publication_feature_issue");
    let post_feature_pr = parse_marker_usize("r49_review_post_publication_feature_pr");

    assert_eq!(baseline_open_issue_count, 1);
    assert_eq!(baseline_open_milestone_count, 1);
    assert_eq!(baseline_ignored_count, 12);

    assert_eq!(post_open_issue_count, 0);
    assert_eq!(post_open_milestone_count, 0);
    assert_eq!(post_ignored_count, 12);
    assert_eq!(post_feature_issue, 5499);
    assert_eq!(post_feature_pr, 5500);

    assert!(post_open_issue_count <= baseline_open_issue_count);
    assert!(post_open_milestone_count <= baseline_open_milestone_count);
}
