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
fn doc_contains_classification_redaction_compliance_contract_lane() {
    assert!(DOC.contains("## Classification/Redaction Compliance Contract Lane"));
    assert!(DOC.contains("classification_redaction_lane_contract.py"));
    assert!(DOC.contains("run_classification_redaction_contract_lane.sh"));
    assert!(DOC.contains("check_classification_redaction_policy.sh"));
    assert!(DOC.contains("classification_redaction_policy_contract.py"));
    assert!(DOC.contains("kamn.compliance.classification-redaction-report.v1"));
    assert!(DOC.contains("classification_redaction_reason_codes:GO:v1"));
    assert!(DOC.contains("classification_redaction_reason_codes:NO-GO:v1"));
}

#[test]
fn regression_requires_legal_hold_precedence_guard_marker() {
    // Regression: #732
    assert!(DOC.contains(
        "legal-hold bypass attempts and tampered DSAR evidence force `NO-GO` (`Regression: #732`)."
    ));
}

#[test]
fn regression_requires_classification_redaction_fail_closed_marker() {
    // Regression: #914
    assert!(DOC.contains(
        "classification/redaction contract drift must fail closed (`Regression: #914`)."
    ));
}

#[test]
fn regression_requires_classification_redaction_policy_wrapper_marker() {
    // Regression: #1222
    assert!(DOC.contains("classification_redaction_policy_contract.py"));
    assert!(DOC.contains("Regression: #1222"));
}

#[test]
fn regression_requires_classification_redaction_lane_wrapper_marker() {
    // Regression: #1226
    assert!(DOC.contains("classification_redaction_lane_contract.py"));
    assert!(DOC.contains("Regression: #1226"));
}
