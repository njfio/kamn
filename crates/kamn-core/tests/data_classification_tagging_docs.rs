const DOC: &str = include_str!("../../../docs/foundation/data-classification-tagging.md");

#[test]
fn doc_contains_classification_core_scope() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("Added `DataClassificationEngine`"));
    assert!(DOC.contains("Added required-tag enforcement by classification level."));
}

#[test]
fn doc_contains_dsar_legal_hold_evidence_contract() {
    assert!(DOC.contains("## DSAR/Export/Erasure Legal-Hold Evidence Contract"));
    assert!(DOC.contains("generate_dsar_legal_hold_evidence_bundle.sh"));
    assert!(DOC.contains("check_dsar_legal_hold_policy.sh"));
    assert!(DOC.contains("run_dsar_legal_hold_contract_lane.sh"));
    assert!(DOC.contains("run_dsar_legal_hold_deep_lane.sh"));
    assert!(DOC.contains("run_dsar_legal_hold_matrix.py"));
    assert!(DOC.contains("fixtures/compliance_dsar/legal_hold_precedence_cases.json"));
}

#[test]
fn regression_requires_legal_hold_precedence_guard_marker() {
    // Regression: #732
    assert!(DOC.contains(
        "legal-hold bypass attempts and tampered DSAR evidence force `NO-GO` (`Regression: #732`)."
    ));
}
