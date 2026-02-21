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
    assert!(DOC.contains("r49_review_baseline_branch_remote_head_count=50"));
    assert!(DOC.contains("r49_review_baseline_open_issue_count=1"));
    assert!(DOC.contains("r49_review_baseline_open_milestone_count=1"));
    assert!(DOC.contains("r49_review_ignored_test_inventory_count=12"));
    assert!(DOC.contains("r49_review_ignored_test_periodic_review_status=completed"));
    assert!(DOC.contains("r49_review_milestone_closure_wave_closed_ids_csv=94,95,96,97"));
}

#[test]
fn integration_r49_review_marker_consistency() {
    let open_issue_count = parse_marker_usize("r49_review_baseline_open_issue_count");
    let open_milestone_count = parse_marker_usize("r49_review_baseline_open_milestone_count");
    let ignored_count = parse_marker_usize("r49_review_ignored_test_inventory_count");
    let branch_count = parse_marker_usize("r49_review_baseline_branch_remote_head_count");

    assert_eq!(open_issue_count, 1);
    assert_eq!(open_milestone_count, 1);
    assert_eq!(ignored_count, 12);
    assert!(branch_count >= 1);
}
