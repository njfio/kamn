use crate::support::{
    data_layer_pg_project_m5_embedding_insert_operation,
    data_layer_pg_project_m5_similarity_search_operation, fixture_m5_embedding_record,
    DataLayerM5SemanticQuery, DataLayerPgM5PgvectorConfig, DataLayerPgM5SimilaritySearchRequest,
    DataLayerPgOperationKind, DataLayerPgRepositoryBridgeError,
    DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE,
    DATA_LAYER_PG_PGVECTOR_EXTENSION_UNAVAILABLE_REASON_CODE,
};

macro_rules! assert_insert_descriptor {
    ($descriptor:expr) => {
        assert_eq!(
            $descriptor.kind,
            DataLayerPgOperationKind::InsertEmbeddingVector
        );
        assert!($descriptor.sql.starts_with("INSERT INTO embeddings"));
        assert_insert_bind_markers(&$descriptor.bind_markers);
    };
}

macro_rules! assert_search_descriptor {
    ($descriptor:expr) => {
        assert_eq!(
            $descriptor.kind,
            DataLayerPgOperationKind::SearchEmbeddingVectors
        );
        assert!($descriptor
            .sql
            .contains("ORDER BY vector_plaintext <=> $2::vector"));
        assert_eq!(
            $descriptor.bind_markers,
            vec!["owner_did", "query_vector", "limit"]
        );
    };
}

#[test]
fn spec_c05_m5_pgvector_projection_is_deterministic_for_insert_and_search() {
    let record = fixture_m5_embedding_record();
    let insert_descriptor = data_layer_pg_project_m5_embedding_insert_operation(
        &record,
        "kamn:did:agent:agent-1",
        enabled_pgvector_config(3),
    )
    .expect("valid m5 embedding should project pgvector insert descriptor");
    assert_insert_descriptor!(insert_descriptor);

    let search_descriptor = data_layer_pg_project_m5_similarity_search_operation(
        similarity_search_request(),
        enabled_pgvector_config(3),
    )
    .expect("valid semantic query should project pgvector similarity descriptor");
    assert_search_descriptor!(search_descriptor);
}

#[test]
fn spec_c06_m5_pgvector_projection_fails_closed_for_extension_and_dimension_mismatch() {
    let record = fixture_m5_embedding_record();
    let unavailable_error = data_layer_pg_project_m5_embedding_insert_operation(
        &record,
        "kamn:did:agent:agent-1",
        disabled_pgvector_config(),
    )
    .expect_err("disabled pgvector extension should fail closed");
    match unavailable_error {
        DataLayerPgRepositoryBridgeError::PgvectorExtensionUnavailable { reason_code } => {
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_PGVECTOR_EXTENSION_UNAVAILABLE_REASON_CODE
            );
        }
        other => panic!("unexpected extension error variant: {other:?}"),
    }

    let mismatch_error = data_layer_pg_project_m5_similarity_search_operation(
        similarity_search_request(),
        enabled_pgvector_config(4),
    )
    .expect_err("dimension mismatch should fail closed");
    assert_dimension_mismatch(mismatch_error);
}

fn similarity_search_request() -> DataLayerPgM5SimilaritySearchRequest {
    DataLayerPgM5SimilaritySearchRequest {
        requester_did: "kamn:did:agent:agent-1".to_owned(),
        query: DataLayerM5SemanticQuery {
            owner_did: "kamn:did:owner:owner-1".to_owned(),
            query_vector: vec![0.1, 0.2, 0.3],
            limit: Some(10),
        },
    }
}

fn assert_insert_bind_markers(markers: &[&str]) {
    assert_eq!(
        markers,
        [
            "embedding_id",
            "message_id",
            "owner_did",
            "agent_did",
            "model_id",
            "vector_plaintext",
            "vector_dimensions",
            "created_at_epoch_seconds",
        ]
    );
}

fn assert_dimension_mismatch(error: DataLayerPgRepositoryBridgeError) {
    match error {
        DataLayerPgRepositoryBridgeError::PgvectorDimensionMismatch {
            reason_code,
            expected,
            found,
        } => {
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE
            );
            assert_eq!(expected, 4);
            assert_eq!(found, 3);
        }
        other => panic!("unexpected mismatch error variant: {other:?}"),
    }
}

fn disabled_pgvector_config() -> DataLayerPgM5PgvectorConfig {
    DataLayerPgM5PgvectorConfig::new(false, 3)
        .expect("disabled pgvector config should still construct")
}

fn enabled_pgvector_config(dimensions: usize) -> DataLayerPgM5PgvectorConfig {
    DataLayerPgM5PgvectorConfig::new(true, dimensions)
        .expect("enabled pgvector config should construct")
}
