use kamn_core::{
    DataLayerM5EmbeddingPrivacyMode, DataLayerM5EmbeddingRegistry, DataLayerM5SemanticQuery,
    DataLayerM5VectorIntegrationError,
    DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE,
};

use super::support::vector_input;

#[test]
fn spec_c03_semantic_query_is_owner_scoped_and_ranked_deterministically() {
    let mut registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn);
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
        Err(DataLayerM5VectorIntegrationError::SemanticQueryUnavailable {
            reason_code: DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE,
        })
    ));
}

#[test]
fn spec_c13_semantic_query_accepts_canonical_equivalent_owner_did() {
    let mut registry =
        DataLayerM5EmbeddingRegistry::new(DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn);
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
