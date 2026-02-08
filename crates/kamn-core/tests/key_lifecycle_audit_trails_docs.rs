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
