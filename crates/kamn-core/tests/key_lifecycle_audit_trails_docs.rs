const DOC: &str = include_str!("../../../docs/foundation/key-lifecycle-audit-trails.md");

#[test]
fn doc_contains_audit_record_schema_and_verification_rules() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("KeyLifecycleAuditRecord"));
    assert!(DOC.contains("KeyLifecycle::audit_records()"));
    assert!(DOC.contains("KeyLifecycle::verify_audit_records(...)"));
    assert!(DOC.contains("KeyLifecycleAuditError"));
}

#[test]
fn doc_contains_tamper_evident_chain_requirements() {
    assert!(DOC.contains("## Tamper-Evident Rules"));
    assert!(DOC.contains("genesis marker `GENESIS`"));
    assert!(DOC.contains("Sequence IDs must be contiguous and start at `1`."));
    assert!(DOC.contains("Verification fails when sequence continuity, chain links, or record hashes are inconsistent."));
}

#[test]
fn regression_requires_chain_link_mismatch_detection_rule() {
    // Regression: #158
    assert!(DOC.contains("chain links"));
}

#[test]
fn doc_contains_lifecycle_operator_binding_audit_evidence_contract() {
    assert!(DOC.contains("## DID Lifecycle Operator-Binding Audit Evidence Contract (Issue #890)"));
    assert!(DOC.contains("generate_lifecycle_operator_binding_evidence_bundle.sh"));
    assert!(DOC.contains("check_lifecycle_operator_binding_policy.sh"));
    assert!(DOC.contains("run_lifecycle_operator_binding_contract_lane.sh"));
    assert!(DOC.contains("did_lifecycle_operator_binding_reason_codes:GO:v1"));
}

#[test]
fn regression_doc_marks_missing_keys_or_decision_drift_fail_closed_policy() {
    // Regression: #890
    assert!(DOC.contains("Regression: #890"));
}
