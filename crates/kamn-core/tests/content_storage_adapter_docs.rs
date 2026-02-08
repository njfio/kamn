const DOC: &str = include_str!("../../../docs/foundation/content-storage-adapter.md");

#[test]
fn doc_contains_adapter_contract_scope_and_helpers() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("ContentStorageAdapter"));
    assert!(DOC.contains("InMemoryContentAdapter"));
    assert!(DOC.contains("content_uri_for_cid"));
    assert!(DOC.contains("cid_from_content_uri"));
}

#[test]
fn doc_contains_integrity_and_task_artifact_integration_rules() {
    assert!(DOC.contains("## Integrity Verification Rules"));
    assert!(DOC.contains("IntegrityMismatch"));
    assert!(DOC.contains("## Task Artifact Integration Path"));
    assert!(DOC.contains("TaskArtifactRegistry::integrity_fingerprint"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test content_storage_adapter"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_tamper_detection_behavior() {
    // Regression: #169
    assert!(DOC.contains("Corruption/tampering returns `ContentStorageError::IntegrityMismatch`."));
}
