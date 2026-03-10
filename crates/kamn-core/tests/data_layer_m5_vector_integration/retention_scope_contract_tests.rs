use kamn_core::{
    ContentRetentionClass, DataLayerM5EmbeddingPrivacyMode, DataLayerM5EmbeddingRecordInput,
    DataLayerM5EmbeddingRegistry, DataLayerM5RetentionDueCandidate, DataLayerM5VectorIntegrationError,
    DATA_LAYER_M5_RETENTION_DUE_REASON_CODE,
};

#[test]
fn spec_c11_retention_due_projection_aligns_with_content_lifecycle_windows() {
    let mut registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn);
    append_retention_input(&mut registry, "embed-m5-retention-1", "msg-m5-retention-1", ContentRetentionClass::ShortLived, vec![0.3, 0.4, 0.3], 1_708_300_000);
    append_retention_input(&mut registry, "embed-m5-retention-2", "msg-m5-retention-2", ContentRetentionClass::Compliance, vec![0.2, 0.5, 0.3], 1_708_300_010);

    let due: Vec<DataLayerM5RetentionDueCandidate> = registry
        .retention_due_for_owner("kamn:did:owner:alpha", 1_708_500_000)
        .expect("retention due should succeed");
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].embedding_id, "embed-m5-retention-1");
    assert_eq!(due[0].message_id, "msg-m5-retention-1");
    assert_eq!(due[0].reason_code, DATA_LAYER_M5_RETENTION_DUE_REASON_CODE);

    let invalid_now = registry.retention_due_for_owner("kamn:did:owner:alpha", 0);
    assert!(matches!(
        invalid_now,
        Err(DataLayerM5VectorIntegrationError::EmptyField("now_epoch_seconds"))
    ));
}

#[test]
fn spec_c12_retention_due_accepts_canonical_equivalent_owner_did() {
    let mut registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn);
    append_retention_input(
        &mut registry,
        "embed-m5-retention-canonical",
        "msg-m5-retention-canonical",
        ContentRetentionClass::ShortLived,
        vec![0.3, 0.4, 0.3],
        1_708_300_000,
    );

    let due = registry.retention_due_for_owner("  kamn:did:owner:alpha  ", 1_708_500_000);
    assert!(
        due.is_ok(),
        "canonical-equivalent owner DID should resolve retention owner scope"
    );
}

fn append_retention_input(
    registry: &mut DataLayerM5EmbeddingRegistry,
    embedding_id: &str,
    message_id: &str,
    retention_class: ContentRetentionClass,
    vector: Vec<f32>,
    created_at_epoch_seconds: u64,
) {
    registry
        .append(DataLayerM5EmbeddingRecordInput {
            embedding_id: embedding_id.to_owned(),
            message_id: message_id.to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha".to_owned(),
            retention_class,
            model_id: "text-embedding-3-large".to_owned(),
            vector_encrypted: vec![0xde, 0xad, 0xbe, 0xef],
            vector_plaintext: Some(vector),
            created_at_epoch_seconds,
        })
        .expect("retention embedding should append");
}
