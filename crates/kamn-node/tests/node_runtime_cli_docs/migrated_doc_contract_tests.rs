use self::migrated_doc_cases::MIGRATED_DOC_CONTRACT_CASES;

#[path = "migrated_doc_contract_tests/migrated_doc_cases.rs"]
mod migrated_doc_cases;

#[test]
fn functional_migrated_doc_contract_cases_require_markers() {
    for case in MIGRATED_DOC_CONTRACT_CASES {
        for marker in case.required_markers {
            assert!(
                case.document.contains(marker),
                "missing marker in {} for case {}: {}",
                case.document_label,
                case.case_id,
                marker
            );
        }
    }
}

#[test]
fn regression_migrated_doc_contract_case_inventory_remains_stable() {
    assert_eq!(MIGRATED_DOC_CONTRACT_CASES.len(), 13);
    assert!(MIGRATED_DOC_CONTRACT_CASES
        .iter()
        .all(|case| !case.required_markers.is_empty()));
    let total_marker_count: usize = MIGRATED_DOC_CONTRACT_CASES
        .iter()
        .map(|case| case.required_markers.len())
        .sum();
    assert_eq!(total_marker_count, 67);
}
