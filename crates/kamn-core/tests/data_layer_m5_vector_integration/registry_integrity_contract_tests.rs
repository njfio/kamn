use kamn_core::{
    DataLayerM5EmbeddingPrivacyMode, DataLayerM5EmbeddingRegistry,
    DataLayerM5VectorIntegrationError, DATA_LAYER_M5_INVALID_AGENT_DID_REASON_CODE,
};

use super::support::vector_input;

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
fn spec_c10_agent_did_validation_uses_canonical_parser_and_fails_closed() {
    let mut registry = DataLayerM5EmbeddingRegistry::new(
        DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn,
    );
    let invalid_agent = registry.append(vector_input(
        "embed-m5-invalid-agent",
        "msg-m5-invalid-agent",
        "kamn:did:owner:alpha",
        "kamn:did:owner:not-an-agent",
        Some(vec![0.4, 0.4, 0.2]),
    ));
    assert!(matches!(
        invalid_agent,
        Err(DataLayerM5VectorIntegrationError::InvalidAgentDid {
            reason_code: DATA_LAYER_M5_INVALID_AGENT_DID_REASON_CODE,
            ..
        })
    ));
}
