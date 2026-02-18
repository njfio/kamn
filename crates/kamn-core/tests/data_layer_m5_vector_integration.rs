use kamn_core::{
    DataLayerM5AnomalyDecision, DataLayerM5AnomalyEvaluationInput, DataLayerM5EmbeddingPrivacyMode,
    DataLayerM5EmbeddingRecordInput, DataLayerM5EmbeddingRegistry, DataLayerM5SemanticQuery,
    DataLayerM5VectorIntegrationError,
};

fn vector_input(
    embedding_id: &str,
    message_id: &str,
    owner_did: &str,
    agent_did: &str,
    vector_plaintext: Option<Vec<f32>>,
) -> DataLayerM5EmbeddingRecordInput {
    DataLayerM5EmbeddingRecordInput {
        embedding_id: embedding_id.to_owned(),
        message_id: message_id.to_owned(),
        owner_did: owner_did.to_owned(),
        agent_did: agent_did.to_owned(),
        model_id: "text-embedding-3-large".to_owned(),
        vector_encrypted: vec![0xde, 0xad, 0xbe, 0xef],
        vector_plaintext,
        created_at_epoch_seconds: 1_708_300_000,
    }
}

#[test]
fn spec_c01_embedding_registry_append_is_deterministic_and_hash_chained() {
    let mut registry_a = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    let mut registry_b = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );

    let input = vector_input(
        "embed-m5-1",
        "msg-m5-1",
        "kamn:did:owner:alpha",
        "kamn:did:agent:alpha",
        Some(vec![0.1, 0.2, 0.3]),
    );
    let record_a = registry_a
        .append(input.clone())
        .expect("append should succeed for registry A");
    let record_b = registry_b
        .append(input)
        .expect("append should succeed for registry B");

    assert_eq!(record_a.record_hash, record_b.record_hash);
    assert!(record_a.record_hash.starts_with("sha256:"));
    registry_a
        .verify_owner_integrity("kamn:did:owner:alpha")
        .expect("integrity check should pass");
}

#[test]
fn spec_c02_duplicate_embedding_id_is_rejected_fail_closed() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    let input = vector_input(
        "embed-m5-dup",
        "msg-m5-dup",
        "kamn:did:owner:alpha",
        "kamn:did:agent:alpha",
        Some(vec![0.3, 0.2, 0.1]),
    );

    registry
        .append(input.clone())
        .expect("first append should succeed");
    let duplicate = registry.append(input);
    assert!(matches!(
        duplicate,
        Err(DataLayerM5VectorIntegrationError::DuplicateEmbeddingId(_))
    ));
}

#[test]
fn spec_c03_semantic_query_is_owner_scoped_and_ranked_deterministically() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    registry
        .append(vector_input(
            "embed-m5-a",
            "msg-m5-a",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![1.0, 0.0, 0.0]),
        ))
        .expect("append a should succeed");
    registry
        .append(vector_input(
            "embed-m5-b",
            "msg-m5-b",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![0.8, 0.2, 0.0]),
        ))
        .expect("append b should succeed");
    registry
        .append(vector_input(
            "embed-m5-other-owner",
            "msg-m5-other-owner",
            "kamn:did:owner:beta",
            "kamn:did:agent:beta",
            Some(vec![1.0, 0.0, 0.0]),
        ))
        .expect("append other owner should succeed");

    let results = registry
        .semantic_query(DataLayerM5SemanticQuery {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            query_vector: vec![1.0, 0.0, 0.0],
            limit: Some(2),
        })
        .expect("semantic query should succeed");

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].message_id, "msg-m5-a");
    assert_eq!(results[1].message_id, "msg-m5-b");
}

#[test]
fn spec_c04_owner_side_encrypted_mode_rejects_server_side_semantic_query() {
    let mut registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted);
    registry
        .append(vector_input(
            "embed-m5-enc",
            "msg-m5-enc",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            None,
        ))
        .expect("owner-side encrypted append should succeed");

    let denied = registry.semantic_query(DataLayerM5SemanticQuery {
        owner_did: "kamn:did:owner:alpha".to_owned(),
        query_vector: vec![1.0, 0.0, 0.0],
        limit: Some(5),
    });
    assert!(matches!(
        denied,
        Err(
            DataLayerM5VectorIntegrationError::SemanticQueryUnavailable {
                reason_code: "m5_vector_owner_side_query_requires_local_index",
            }
        )
    ));
}

#[test]
fn spec_c05_anomaly_threshold_detection_uses_centroid_distance_rules() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    registry
        .append(vector_input(
            "embed-m5-h1",
            "msg-m5-h1",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![1.0, 0.0, 0.0]),
        ))
        .expect("append history 1 should succeed");
    registry
        .append(vector_input(
            "embed-m5-h2",
            "msg-m5-h2",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![0.95, 0.05, 0.0]),
        ))
        .expect("append history 2 should succeed");
    registry
        .append(vector_input(
            "embed-m5-h3",
            "msg-m5-h3",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![0.9, 0.1, 0.0]),
        ))
        .expect("append history 3 should succeed");

    let decision = registry
        .evaluate_agent_anomaly(DataLayerM5AnomalyEvaluationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:alpha".to_owned(),
            candidate_vector: vec![0.0, 1.0, 0.0],
            lookback_window: Some(3),
            anomaly_distance_threshold: 0.4,
        })
        .expect("anomaly evaluation should succeed");

    assert!(matches!(
        decision,
        DataLayerM5AnomalyDecision::Anomalous {
            reason_code: "m5_vector_anomaly_threshold_exceeded",
            ..
        }
    ));
}
