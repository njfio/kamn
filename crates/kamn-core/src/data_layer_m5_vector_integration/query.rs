use super::models::*;
use super::support::{cosine_similarity, owner_vector_dimensions, parse_kamn_did, resolve_limit, validate_vector};

impl DataLayerM5EmbeddingRegistry {
    /// Executes deterministic owner-scoped semantic top-k ranking.
    pub fn semantic_query(
        &self,
        query: DataLayerM5SemanticQuery,
    ) -> Result<Vec<DataLayerM5SemanticQueryResult>, DataLayerM5VectorIntegrationError> {
        let owner_did = parse_kamn_did(query.owner_did.as_str())?;
        let query_vector = validate_vector(query.query_vector, "query_vector")?;
        let limit = resolve_limit(query.limit)?;
        require_query_mode(self.privacy_mode)?;
        let owner_records = owner_records(self, owner_did.as_str())?;
        validate_query_dimensions(owner_records, query_vector.len())?;
        let mut rows = score_rows(owner_records, query_vector.as_slice())?;
        rows.sort_by(|left, right| {
            right
                .similarity_score
                .total_cmp(&left.similarity_score)
                .then_with(|| left.message_id.cmp(&right.message_id))
                .then_with(|| left.embedding_id.cmp(&right.embedding_id))
        });
        rows.truncate(rows.len().min(limit));
        Ok(rows)
    }
}

fn require_query_mode(
    privacy_mode: DataLayerM5EmbeddingPrivacyMode,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    if privacy_mode == DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted {
        return Err(DataLayerM5VectorIntegrationError::SemanticQueryUnavailable {
            reason_code: DATA_LAYER_M5_OWNER_SIDE_QUERY_REQUIRES_LOCAL_INDEX_REASON_CODE,
        });
    }
    Ok(())
}

fn owner_records<'a>(
    registry: &'a DataLayerM5EmbeddingRegistry,
    owner_did: &str,
) -> Result<&'a [DataLayerM5EmbeddingRecord], DataLayerM5VectorIntegrationError> {
    registry.records_by_owner.get(owner_did).map(Vec::as_slice).ok_or_else(|| {
        DataLayerM5VectorIntegrationError::OwnerNotFound {
            owner_did: owner_did.to_owned(),
        }
    })
}

fn validate_query_dimensions(
    owner_records: &[DataLayerM5EmbeddingRecord],
    query_dimensions: usize,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    let expected_dimensions = owner_vector_dimensions(owner_records).ok_or(
        DataLayerM5VectorIntegrationError::SemanticQueryUnavailable {
            reason_code: DATA_LAYER_M5_PLAINTEXT_INDEX_MISSING_FOR_OWNER_SCOPE_REASON_CODE,
        },
    )?;
    if query_dimensions != expected_dimensions {
        return Err(DataLayerM5VectorIntegrationError::InvalidVectorDimensions {
            expected: expected_dimensions,
            found: query_dimensions,
        });
    }
    Ok(())
}

fn score_rows(
    owner_records: &[DataLayerM5EmbeddingRecord],
    query_vector: &[f32],
) -> Result<Vec<DataLayerM5SemanticQueryResult>, DataLayerM5VectorIntegrationError> {
    owner_records
        .iter()
        .filter_map(|record| record.vector_plaintext.as_ref().map(|vector| (record, vector)))
        .map(|(record, vector)| {
            let similarity = cosine_similarity(query_vector, vector.as_slice())?;
            Ok(DataLayerM5SemanticQueryResult {
                embedding_id: record.embedding_id.clone(),
                message_id: record.message_id.clone(),
                similarity_score: similarity,
                cosine_distance: 1.0 - similarity,
            })
        })
        .collect()
}
