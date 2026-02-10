const DOC: &str = include_str!("../../../docs/foundation/redaction-tombstones.md");

#[test]
fn doc_contains_redaction_scope_and_validation() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("RedactionComplianceEngine"));
    assert!(DOC.contains("## Local Validation"));
}

#[test]
fn doc_contains_classification_redaction_compliance_contract_lane() {
    assert!(DOC.contains("## Classification/Redaction Compliance Contract Lane"));
    assert!(DOC.contains("classification_redaction_lane_contract.py"));
    assert!(DOC.contains("run_classification_redaction_contract_lane.sh"));
    assert!(DOC.contains("classification_redaction_contract_lane_contract.py"));
    assert!(DOC.contains("check_classification_redaction_policy.sh"));
    assert!(DOC.contains("classification_redaction_policy_contract.py"));
    assert!(DOC.contains("kamn.compliance.classification-redaction-report.v1"));
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

#[test]
fn regression_requires_classification_redaction_contract_lane_wrapper_marker() {
    // Regression: #1230
    assert!(DOC.contains("classification_redaction_contract_lane_contract.py"));
    assert!(DOC.contains("Regression: #1230"));
}
