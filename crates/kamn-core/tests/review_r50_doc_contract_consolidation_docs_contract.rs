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

#[test]
fn functional_r50_doc_contract_consolidation_markers_present() {
    assert!(DOC.contains(
        "r50_review_doc_contract_consolidation_schema_version=kamn.review.doc-contract-suite-consolidation-plan.v1"
    ));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_baseline_test_file_count=82"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_target_test_file_cap=74"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_required_reduction=8"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_tranche_count=2"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_min_reduction_per_tranche=4"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_issue_cap_per_tranche=2"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_target_release=r53"));
    assert!(DOC.contains("r50_review_doc_contract_consolidation_status=active"));
    assert!(DOC.contains(
        "Doc-contract consolidation contract active (R50.19) with 2 tranches at minimum 4 reductions each toward <=74 files."
    ));
}

#[test]
fn integration_r50_doc_contract_consolidation_markers_are_consistent() {
    let baseline =
        parse_marker_usize("r50_review_doc_contract_consolidation_baseline_test_file_count");
    let target_cap =
        parse_marker_usize("r50_review_doc_contract_consolidation_target_test_file_cap");
    let required_reduction =
        parse_marker_usize("r50_review_doc_contract_consolidation_required_reduction");
    let tranche_count = parse_marker_usize("r50_review_doc_contract_consolidation_tranche_count");
    let min_reduction_per_tranche =
        parse_marker_usize("r50_review_doc_contract_consolidation_min_reduction_per_tranche");
    let issue_cap_per_tranche =
        parse_marker_usize("r50_review_doc_contract_consolidation_issue_cap_per_tranche");

    assert!(
        baseline > target_cap,
        "baseline must be greater than target cap"
    );
    assert_eq!(baseline.saturating_sub(target_cap), required_reduction);
    assert!(tranche_count > 0, "tranche count must be positive");
    assert!(
        tranche_count.saturating_mul(min_reduction_per_tranche) >= required_reduction,
        "tranche plan must cover required reduction"
    );
    assert!(
        issue_cap_per_tranche <= 2,
        "issue cap per tranche must remain tightly bounded"
    );
}
