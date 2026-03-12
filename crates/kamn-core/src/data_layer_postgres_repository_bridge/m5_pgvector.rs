use crate::{DataLayerM5EmbeddingRecord, DataLayerPgRepositoryBridgeError};

use super::{
    build_requester_session, validate_non_empty, validate_owner_did, validate_pgvector_extension,
    DataLayerPgM5PgvectorConfig, DataLayerPgM5SimilaritySearchRequest, DataLayerPgOperationKind,
    DataLayerPgSqlOperation, DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT,
    DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE,
};

pub fn data_layer_pg_project_m5_embedding_insert_operation(
    record: &DataLayerM5EmbeddingRecord,
    requester_did: &str,
    config: DataLayerPgM5PgvectorConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_pgvector_extension(config)?;
    validate_embedding_record(record)?;
    validate_vector(record.vector_plaintext.as_ref(), config.dimensions)?;
    Ok(build_embedding_insert_operation(requester_did)?)
}

pub fn data_layer_pg_project_m5_similarity_search_operation(
    request: DataLayerPgM5SimilaritySearchRequest,
    config: DataLayerPgM5PgvectorConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_pgvector_extension(config)?;
    validate_similarity_request(&request, config.dimensions)?;
    let limit = resolve_vector_limit(request.query.limit)?;
    Ok(build_similarity_search_operation(
        request.requester_did.as_str(),
        limit,
    )?)
}

fn validate_embedding_record(
    record: &DataLayerM5EmbeddingRecord,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    validate_non_empty(record.embedding_id.as_str(), "embedding_id")?;
    validate_non_empty(record.message_id.as_str(), "message_id")?;
    validate_non_empty(record.owner_did.as_str(), "owner_did")?;
    validate_non_empty(record.agent_did.as_str(), "agent_did")?;
    validate_non_empty(record.model_id.as_str(), "model_id")?;
    validate_owner_did(record.owner_did.as_str())
}

fn validate_vector(
    vector: Option<&Vec<f32>>,
    expected: usize,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    let found = vector.map_or(0, Vec::len);
    if found != expected {
        return Err(vector_mismatch(expected, found));
    }
    Ok(())
}

fn validate_similarity_request(
    request: &DataLayerPgM5SimilaritySearchRequest,
    expected: usize,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    validate_non_empty(request.query.owner_did.as_str(), "owner_did")?;
    validate_owner_did(request.query.owner_did.as_str())?;
    if request.query.query_vector.is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField("query_vector"));
    }
    if request.query.query_vector.len() != expected {
        return Err(vector_mismatch(expected, request.query.query_vector.len()));
    }
    Ok(())
}

fn resolve_vector_limit(limit: Option<usize>) -> Result<usize, DataLayerPgRepositoryBridgeError> {
    let limit = limit.unwrap_or(DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT);
    if limit == 0 || limit > DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT {
        return Err(DataLayerPgRepositoryBridgeError::InvalidSearchLimit {
            requested: limit as u32,
            max_allowed: DATA_LAYER_PG_MAX_VECTOR_SEARCH_LIMIT as u32,
        });
    }
    Ok(limit)
}

fn build_embedding_insert_operation(
    requester_did: &str,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::InsertEmbeddingVector,
        sql: embedding_insert_sql(),
        bind_markers: embedding_insert_bind_markers(),
        session: build_requester_session(requester_did)?,
    })
}

fn build_similarity_search_operation(
    requester_did: &str,
    _limit: usize,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::SearchEmbeddingVectors,
        sql: similarity_search_sql(),
        bind_markers: vec!["owner_did", "query_vector", "limit"],
        session: build_requester_session(requester_did)?,
    })
}

fn embedding_insert_sql() -> String {
    "INSERT INTO embeddings (embedding_id, message_id, owner_did, agent_did, model_id, vector_plaintext, vector_dimensions, created_at) VALUES ($1::uuid, $2::uuid, $3, $4, $5, $6::vector, $7, to_timestamp($8));".to_owned()
}

fn embedding_insert_bind_markers() -> Vec<&'static str> {
    vec![
        "embedding_id",
        "message_id",
        "owner_did",
        "agent_did",
        "model_id",
        "vector_plaintext",
        "vector_dimensions",
        "created_at_epoch_seconds",
    ]
}

fn similarity_search_sql() -> String {
    "SELECT embedding_id, message_id, owner_did, agent_did, model_id, vector_dimensions, vector_plaintext <=> $2::vector AS cosine_distance FROM embeddings WHERE owner_did = $1 ORDER BY vector_plaintext <=> $2::vector ASC LIMIT $3;".to_owned()
}

fn vector_mismatch(expected: usize, found: usize) -> DataLayerPgRepositoryBridgeError {
    DataLayerPgRepositoryBridgeError::PgvectorDimensionMismatch {
        reason_code: DATA_LAYER_PG_PGVECTOR_DIMENSION_MISMATCH_REASON_CODE,
        expected,
        found,
    }
}
