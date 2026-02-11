const RUNTIME_COMMIT_SRC: &str = include_str!("../src/kolme_runtime_commit.rs");

#[test]
fn unit_runtime_commit_extraction_boundary_removes_local_finality_glue_wrappers() {
    assert!(!RUNTIME_COMMIT_SRC.contains("fn parse_receipt_finality("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn map_extracted_receipt_finality("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn lifecycle_state_for_finality("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn lifecycle_state_label("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn commit_finality_label("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn deterministic_idempotency_key("));
    assert!(!RUNTIME_COMMIT_SRC.contains("fn deterministic_commit_id("));
}

#[test]
fn regression_runtime_commit_extraction_boundary_keeps_direct_helper_delegation() {
    // Regression: #1790
    assert!(RUNTIME_COMMIT_SRC.contains("parse_kolme_commit_receipt_finality("));
    assert!(RUNTIME_COMMIT_SRC.contains("commit_finality_from_receipt_finality_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("lifecycle_state_for_finality_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("lifecycle_state_label_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("commit_finality_label_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("deterministic_runtime_commit_idempotency_key_contract("));
    assert!(RUNTIME_COMMIT_SRC.contains("deterministic_runtime_commit_id_contract("));
}
