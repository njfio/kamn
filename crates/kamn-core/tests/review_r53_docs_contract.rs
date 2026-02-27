use std::collections::BTreeMap;

const DOC_R53: &str = include_str!("../../../docs/review/gaps-and-issues-r53.md");
const DOC_R54: &str = include_str!("../../../docs/review/gaps-and-issues-r54.md");
const DOC_R55: &str = include_str!("../../../docs/review/gaps-and-issues-r55.md");
const DOC_R56: &str = include_str!("../../../docs/review/gaps-and-issues-r56.md");
const REVIEW_MARKER_README: &str = include_str!("../../../docs/review/README.md");
const REVIEW_FREEZE_MANIFEST: &str =
    include_str!("../../../docs/review/review-document-freeze.manifest");

fn parse_markers(doc: &str) -> BTreeMap<String, String> {
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

fn parse_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn assert_unresolved_closure_contract(
    markers: &BTreeMap<String, String>,
    release_prefix: &str,
    expected_schema_version: &str,
) {
    let schema_key = format!("{release_prefix}_review_unresolved_closure_schema_version");
    let total_key = format!("{release_prefix}_review_unresolved_total_item_count");
    let resolved_key = format!("{release_prefix}_review_unresolved_resolved_item_count");
    let closure_status_key = format!("{release_prefix}_review_unresolved_closure_status");

    assert_eq!(
        parse_marker_value(markers, schema_key.as_str()),
        expected_schema_version
    );

    let total = parse_marker_usize(markers, total_key.as_str());
    let resolved = parse_marker_usize(markers, resolved_key.as_str());
    let closure_status = parse_marker_value(markers, closure_status_key.as_str());

    assert!(
        resolved <= total,
        "resolved count cannot exceed total unresolved count"
    );
    match closure_status {
        "all_resolved" => assert_eq!(
            resolved, total,
            "all_resolved must report resolved == total"
        ),
        "partial_resolution_with_active_reduction" => {
            assert!(
                resolved < total,
                "partial resolution must leave unresolved items"
            );
        }
        other => panic!("unsupported unresolved closure status: {other}"),
    }
}

#[test]
fn functional_review_readme_declares_core_review_governance_contracts() {
    assert!(REVIEW_MARKER_README.contains(
        "governance_feature_activity_ratio_schema_version=kamn.review.governance-feature-activity-ratio.v1"
    ));
    assert!(REVIEW_MARKER_README.contains(
        "review_snapshot_semantics_policy_schema_version=kamn.review.snapshot-semantics-policy.v1"
    ));
    assert!(REVIEW_MARKER_README.contains(
        "review_document_freeze_policy_schema_version=kamn.review.review-document-freeze-policy.v1"
    ));
    assert!(REVIEW_MARKER_README.contains(
        "review_document_immutability_schema_version=kamn.review.review-document-immutability-policy.v1"
    ));
    assert!(REVIEW_MARKER_README.contains(
        "review_document_freeze_manifest_schema_version=kamn.review.review-document-freeze-manifest.v1"
    ));
}

#[test]
fn integration_r54_plus_unresolved_closure_markers_are_consistent() {
    let r54 = parse_markers(DOC_R54);
    assert_unresolved_closure_contract(&r54, "r54", "kamn.review.unresolved-item-closure.v1");

    let r55 = parse_markers(DOC_R55);
    assert_unresolved_closure_contract(&r55, "r55", "kamn.review.unresolved-item-closure.v1");

    let r56 = parse_markers(DOC_R56);
    assert_unresolved_closure_contract(&r56, "r56", "kamn.review.unresolved-item-closure.v2");
}

#[test]
fn integration_review_document_freeze_manifest_tracks_expected_review_files() {
    let markers = parse_markers(REVIEW_FREEZE_MANIFEST);
    assert_eq!(
        parse_marker_value(&markers, "review_document_freeze_manifest_schema_version"),
        "kamn.review.review-document-freeze-manifest.v1"
    );

    let entries = parse_csv(parse_marker_value(
        &markers,
        "review_document_freeze_entries_csv",
    ));
    let expected_entries = [
        "gaps-and-issues-r51.md",
        "gaps-and-issues-r52.md",
        "gaps-and-issues-r53.md",
        "gaps-and-issues-r54.md",
        "gaps-and-issues-r55.md",
        "gaps-and-issues-r56.md",
    ];
    assert_eq!(entries.len(), expected_entries.len());
    for expected in expected_entries {
        assert!(entries.iter().any(|entry| entry == expected));
        let release = expected
            .trim_start_matches("gaps-and-issues-r")
            .trim_end_matches(".md");
        let line_count_key = format!("r{release}_review_freeze_line_count");
        let fnv_key = format!("r{release}_review_freeze_fnv1a64_hex");
        let tail_key = format!("r{release}_review_freeze_last_non_empty_line");

        assert!(
            markers.contains_key(line_count_key.as_str()),
            "missing freeze line-count marker for {expected}"
        );
        assert!(
            markers.contains_key(fnv_key.as_str()),
            "missing freeze checksum marker for {expected}"
        );
        assert!(
            markers.contains_key(tail_key.as_str()),
            "missing freeze tail-line marker for {expected}"
        );
    }
}

#[test]
fn regression_review_docs_keep_core_post_publication_marker_chain() {
    let r53 = parse_markers(DOC_R53);
    assert_eq!(
        parse_marker_value(
            &r53,
            "r53_review_doc_contract_non_regression_schema_version"
        ),
        "kamn.review.doc-contract-non-regression-ratchet.v1"
    );

    let r54 = parse_markers(DOC_R54);
    assert!(matches!(
        parse_marker_value(&r54, "r54_review_unresolved_closure_status"),
        "all_resolved"
    ));

    let r55 = parse_markers(DOC_R55);
    assert!(matches!(
        parse_marker_value(&r55, "r55_review_unresolved_closure_status"),
        "all_resolved"
    ));

    let r56 = parse_markers(DOC_R56);
    assert!(matches!(
        parse_marker_value(&r56, "r56_review_unresolved_closure_status"),
        "partial_resolution_with_active_reduction"
    ));
    assert_eq!(
        parse_marker_value(&r56, "r56_review_review_document_freeze_status"),
        "enforced"
    );
}

#[test]
fn regression_review_readme_schema_marker_count_is_non_expanding() {
    let schema_marker_count = REVIEW_MARKER_README
        .lines()
        .filter(|line| line.contains("schema_version"))
        .count();
    assert!(
        schema_marker_count <= 22,
        "review README schema marker count expanded beyond baseline cap: {}",
        schema_marker_count
    );
}
