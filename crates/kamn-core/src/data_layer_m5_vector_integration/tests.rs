use super::{
    DataLayerM5EmbeddingPrivacyMode, DataLayerM5EmbeddingRecordInput, DataLayerM5EmbeddingRegistry,
    DataLayerM5SemanticQuery, DataLayerM5VectorIntegrationError,
};
use crate::ContentRetentionClass;

fn fixture_input(
    embedding_id: &str,
    message_id: &str,
    vector_plaintext: Option<Vec<f32>>,
) -> DataLayerM5EmbeddingRecordInput {
    DataLayerM5EmbeddingRecordInput {
        embedding_id: embedding_id.to_owned(),
        message_id: message_id.to_owned(),
        owner_did: "kamn:did:owner:alice".to_owned(),
        agent_did: "kamn:did:agent:alice".to_owned(),
        retention_class: ContentRetentionClass::Standard,
        model_id: "model-1".to_owned(),
        vector_encrypted: vec![0x01, 0x02, 0x03],
        vector_plaintext,
        created_at_epoch_seconds: 1_000,
    }
}

#[test]
fn unit_data_layer_m5_append_and_semantic_query_rank_results() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    registry
        .append(fixture_input("emb-1", "msg-1", Some(vec![1.0, 0.0])))
        .expect("first embedding append should succeed");
    registry
        .append(fixture_input("emb-2", "msg-2", Some(vec![0.0, 1.0])))
        .expect("second embedding append should succeed");

    let results = registry
        .semantic_query(DataLayerM5SemanticQuery {
            owner_did: "kamn:did:owner:alice".to_owned(),
            query_vector: vec![1.0, 0.0],
            limit: Some(2),
        })
        .expect("semantic query should rank plaintext vectors");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].embedding_id, "emb-1");
    assert!(results[0].similarity_score > results[1].similarity_score);
}

#[test]
fn unit_data_layer_m5_owner_side_mode_rejects_plaintext_ingestion() {
    let mut registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted);
    let error = registry
        .append(fixture_input("emb-1", "msg-1", Some(vec![1.0, 0.0])))
        .expect_err("owner-side mode must reject plaintext vectors");
    assert!(matches!(
        error,
        DataLayerM5VectorIntegrationError::PrivacyModeViolation { .. }
    ));
}
