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
fn functional_r45_review_coherence_batching_policy_markers_present() {
    assert!(DOC.contains("### 5.4 Coherence Contract Batching Policy"));
    assert!(DOC.contains(
        "coherence_contract_batching_policy_schema_version=kamn.review.coherence-contract-batching-policy.v1"
    ));
    assert!(DOC.contains("coherence_contract_batching_dimension_baseline=28"));
    assert!(DOC.contains("coherence_contract_batching_issue_baseline=28"));
    assert!(DOC.contains("coherence_contract_batching_target_bundle_count_min=5"));
    assert!(DOC.contains("coherence_contract_batching_target_bundle_count_max=8"));
    assert!(DOC.contains("coherence_contract_batching_target_issue_cap=8"));
    assert!(DOC.contains("coherence_contract_batching_expected_issue_reduction=20"));
}

#[test]
fn integration_r45_review_coherence_batching_policy_markers_are_consistent() {
    let dimension_baseline = parse_marker_usize("coherence_contract_batching_dimension_baseline");
    let issue_baseline = parse_marker_usize("coherence_contract_batching_issue_baseline");
    let bundle_min = parse_marker_usize("coherence_contract_batching_target_bundle_count_min");
    let bundle_max = parse_marker_usize("coherence_contract_batching_target_bundle_count_max");
    let issue_cap = parse_marker_usize("coherence_contract_batching_target_issue_cap");
    let expected_reduction =
        parse_marker_usize("coherence_contract_batching_expected_issue_reduction");

    assert!(bundle_min >= 1, "bundle minimum should be positive");
    assert!(
        bundle_max >= bundle_min,
        "bundle max should be >= bundle minimum"
    );
    assert!(issue_cap >= bundle_max, "issue cap should be >= bundle max");
    assert!(
        dimension_baseline >= issue_cap,
        "dimension baseline should be >= issue cap"
    );
    assert_eq!(
        issue_baseline.saturating_sub(issue_cap),
        expected_reduction,
        "expected issue reduction should match baseline minus target issue cap"
    );
}
