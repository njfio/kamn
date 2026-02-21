const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r45.md");

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
fn functional_r45_review_branch_hygiene_refresh_markers_present() {
    assert!(DOC.contains("### 6.2 Branch Hygiene Refresh Wave (Issue #5424)"));
    assert!(DOC.contains("branch_hygiene_refresh_issue=5424"));
    assert!(DOC.contains("branch_hygiene_refresh_strategy=merged_only"));
    assert!(DOC.contains(
        "branch_hygiene_refresh_evidence_command_pre=git ls-remote --heads origin | wc -l"
    ));
    assert!(DOC.contains(
        "branch_hygiene_refresh_evidence_command_candidates=git branch -r --merged origin/main"
    ));
    assert!(DOC.contains(
        "branch_hygiene_refresh_evidence_command_post=git ls-remote --heads origin | wc -l"
    ));
}

#[test]
fn integration_r45_review_branch_hygiene_refresh_counts_are_consistent() {
    let pre_count = parse_marker_usize("branch_hygiene_remote_branch_count_pre_cleanup");
    let deleted_count = parse_marker_usize("branch_hygiene_remote_branch_count_deleted");
    let post_count = parse_marker_usize("branch_hygiene_remote_branch_count_post_cleanup");
    assert!(
        pre_count >= post_count,
        "post-cleanup count should not exceed pre-cleanup count"
    );
    assert_eq!(
        pre_count.saturating_sub(post_count),
        deleted_count,
        "deleted count should match pre/post delta"
    );
    assert!(
        post_count <= 60,
        "post-cleanup count should satisfy <=60 bounded branch target"
    );
}
