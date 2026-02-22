const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r52.md");
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

#[test]
fn functional_r52_branch_hygiene_reconciliation_markers_present() {
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_post_publication_branch_cleanup_schema_version"));
    assert!(REVIEW_MARKER_README.contains("kamn.review.branch-hygiene-post-publication-cleanup.v1"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_remote_head_count_pre_cleanup=<integer>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_remote_head_count_deleted=<integer>"));
    assert!(REVIEW_MARKER_README
        .contains("r<release>_review_branch_remote_head_count_post_cleanup=<integer>"));

    assert!(DOC.contains(
        "r52_review_post_publication_branch_cleanup_schema_version=kamn.review.branch-hygiene-post-publication-cleanup.v1"
    ));
    assert!(DOC.contains("r52_review_branch_remote_head_count_baseline_snapshot=67"));
    assert!(DOC.contains("r52_review_branch_remote_head_count_pre_cleanup="));
    assert!(DOC.contains("r52_review_branch_remote_head_count_deleted="));
    assert!(DOC.contains("r52_review_branch_remote_head_count_post_cleanup="));
}

#[test]
fn integration_r52_branch_hygiene_reconciliation_counts_are_consistent() {
    let baseline = parse_marker_usize("r52_review_branch_remote_head_count_baseline_snapshot");
    let pre = parse_marker_usize("r52_review_branch_remote_head_count_pre_cleanup");
    let deleted = parse_marker_usize("r52_review_branch_remote_head_count_deleted");
    let post = parse_marker_usize("r52_review_branch_remote_head_count_post_cleanup");

    assert!(
        pre >= post,
        "post-cleanup branch count should not exceed pre-cleanup count"
    );
    assert_eq!(
        pre.saturating_sub(post),
        deleted,
        "deleted count should match pre/post delta"
    );
    assert!(
        post <= baseline,
        "post-cleanup count should not exceed R52 baseline snapshot"
    );
}
