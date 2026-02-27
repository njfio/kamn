use std::collections::BTreeMap;

const DOC: &str = include_str!("../../../docs/review/gaps-and-issues-r52.md");
const REVIEW_MARKER_README: &str = include_str!("../../../docs/review/README.md");

fn parse_bullet_markers(doc: &str) -> BTreeMap<String, String> {
    let mut markers = BTreeMap::new();
    for raw_line in doc.lines() {
        let mut trimmed = raw_line.trim();
        if let Some(value) = trimmed.strip_prefix("- ") {
            trimmed = value.trim();
        }
        if let Some(value) = trimmed
            .strip_prefix('`')
            .and_then(|value| value.strip_suffix('`'))
        {
            trimmed = value.trim();
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        markers.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    markers
}

fn parse_marker_value<'a>(markers: &'a BTreeMap<String, String>, key: &str) -> &'a str {
    markers
        .get(key)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("missing marker {key}"))
}

fn parse_marker_usize(markers: &BTreeMap<String, String>, key: &str) -> usize {
    parse_marker_value(markers, key)
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("marker {key} should be an unsigned integer"))
}

#[test]
fn functional_r52_reconciliation_schemas_are_documented() {
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_post_publication_branch_cleanup_schema_version=kamn.review.branch-hygiene-post-publication-cleanup.v1"
    ));
    assert!(REVIEW_MARKER_README.contains(
        "r<release>_review_post_publication_quality_gate_reconciliation_schema_version=kamn.review.quality-gate-post-publication-reconciliation.v1"
    ));
}

#[test]
fn integration_r52_branch_cleanup_markers_are_consistent() {
    let markers = parse_bullet_markers(DOC);
    assert_eq!(
        parse_marker_value(
            &markers,
            "r52_review_post_publication_branch_cleanup_schema_version",
        ),
        "kamn.review.branch-hygiene-post-publication-cleanup.v1"
    );

    let pre = parse_marker_usize(&markers, "r52_review_branch_remote_head_count_pre_cleanup");
    let deleted = parse_marker_usize(&markers, "r52_review_branch_remote_head_count_deleted");
    let post = parse_marker_usize(&markers, "r52_review_branch_remote_head_count_post_cleanup");

    assert_eq!(
        pre.saturating_sub(deleted),
        post,
        "branch cleanup arithmetic marker invariant must hold"
    );
    assert!(post <= pre);
}

#[test]
fn integration_r52_quality_gate_markers_are_consistent() {
    let markers = parse_bullet_markers(DOC);
    assert_eq!(
        parse_marker_value(
            &markers,
            "r52_review_post_publication_quality_gate_reconciliation_schema_version",
        ),
        "kamn.review.quality-gate-post-publication-reconciliation.v1"
    );

    let workspace_gate = parse_marker_value(
        &markers,
        "r52_review_workspace_quality_gate_status_post_publication",
    );
    assert!(matches!(workspace_gate, "pass" | "fail"));

    let cli_status = parse_marker_value(&markers, "r52_review_cli_compile_status_post_publication");
    assert!(matches!(cli_status, "resolved" | "unresolved"));

    let marker_parse_status = parse_marker_value(
        &markers,
        "r52_review_activity_ratio_marker_parse_status_post_publication",
    );
    assert!(matches!(marker_parse_status, "resolved" | "unresolved"));
}

#[test]
fn regression_r52_quality_gate_resolution_has_not_regressed() {
    let markers = parse_bullet_markers(DOC);
    assert_eq!(
        parse_marker_value(
            &markers,
            "r52_review_workspace_quality_gate_status_post_publication",
        ),
        "pass"
    );
    assert_eq!(
        parse_marker_value(&markers, "r52_review_cli_compile_status_post_publication"),
        "resolved"
    );
    assert_eq!(
        parse_marker_value(
            &markers,
            "r52_review_activity_ratio_marker_parse_status_post_publication",
        ),
        "resolved"
    );
}
