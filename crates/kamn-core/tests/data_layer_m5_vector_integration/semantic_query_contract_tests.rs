use kamn_core::{
    DataLayerM5EmbeddingPrivacyMode, DataLayerM5EmbeddingRegistry, DataLayerM5SemanticQuery,
    DataLayerM5VectorIntegrationError,
    DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE,
};

use super::support::vector_input;

#[test]
fn spec_c03_semantic_query_is_owner_scoped_and_ranked_deterministically() {
    let registry = seeded_semantic_registry();
    let results = run_owner_query(&registry, "kamn:did:owner:alpha", 2)
        .expect("semantic query should succeed");

    assert_ranked_results(results.as_slice(), &["msg-m5-a", "msg-m5-b"]);
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
                reason_code: DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE,
            }
        )
    ));
}

#[test]
fn spec_c13_semantic_query_accepts_canonical_equivalent_owner_did() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    registry
        .append(vector_input(
            "embed-m5-canonical-query",
            "msg-m5-canonical-query",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            Some(vec![1.0, 0.0, 0.0]),
        ))
        .expect("embedding should append");

    let results = registry.semantic_query(DataLayerM5SemanticQuery {
        owner_did: "  kamn:did:owner:alpha  ".to_owned(),
        query_vector: vec![1.0, 0.0, 0.0],
        limit: Some(1),
    });
    assert!(
        results.is_ok(),
        "canonical-equivalent owner DID should resolve semantic query scope"
    );
}

fn seeded_semantic_registry() -> DataLayerM5EmbeddingRegistry {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    for (embedding_id, message_id, owner_did, agent_did, vector) in semantic_entries() {
        append_embedding(
            &mut registry,
            embedding_id,
            message_id,
            owner_did,
            agent_did,
            vector,
        );
    }
    registry
}

fn append_embedding(
    registry: &mut DataLayerM5EmbeddingRegistry,
    embedding_id: &str,
    message_id: &str,
    owner_did: &str,
    agent_did: &str,
    vector: Vec<f32>,
) {
    registry
        .append(vector_input(
            embedding_id,
            message_id,
            owner_did,
            agent_did,
            Some(vector),
        ))
        .expect("semantic test append should succeed");
}

fn semantic_entries() -> [(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    Vec<f32>,
); 3] {
    [
        (
            "embed-m5-a",
            "msg-m5-a",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            vec![1.0, 0.0, 0.0],
        ),
        (
            "embed-m5-b",
            "msg-m5-b",
            "kamn:did:owner:alpha",
            "kamn:did:agent:alpha",
            vec![0.8, 0.2, 0.0],
        ),
        (
            "embed-m5-other-owner",
            "msg-m5-other-owner",
            "kamn:did:owner:beta",
            "kamn:did:agent:beta",
            vec![1.0, 0.0, 0.0],
        ),
    ]
}

fn run_owner_query(
    registry: &DataLayerM5EmbeddingRegistry,
    owner_did: &str,
    limit: usize,
) -> Result<Vec<kamn_core::DataLayerM5SemanticQueryResult>, DataLayerM5VectorIntegrationError> {
    registry.semantic_query(DataLayerM5SemanticQuery {
        owner_did: owner_did.to_owned(),
        query_vector: vec![1.0, 0.0, 0.0],
        limit: Some(limit),
    })
}

fn assert_ranked_results(
    results: &[kamn_core::DataLayerM5SemanticQueryResult],
    expected_message_ids: &[&str],
) {
    let actual: Vec<&str> = results
        .iter()
        .map(|result| result.message_id.as_str())
        .collect();
    assert_eq!(actual, expected_message_ids);
}
