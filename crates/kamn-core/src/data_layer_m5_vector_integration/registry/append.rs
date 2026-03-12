use super::super::models::*;
use super::super::support::{
    compute_embedding_record_hash, owner_vector_dimensions, parse_kamn_did,
    validate_agent_did, validate_non_empty, validate_vector, DataLayerM5RecordHashMaterial,
};
use crate::ContentRetentionClass;

impl DataLayerM5EmbeddingRegistry {
    /// Appends one embedding record under owner scope.
    pub fn append(
        &mut self,
        input: DataLayerM5EmbeddingRecordInput,
    ) -> Result<DataLayerM5EmbeddingRecord, DataLayerM5VectorIntegrationError> {
        let privacy_mode = self.privacy_mode();
        let append_input = prepare_append_input(input, &self.seen_embedding_ids, privacy_mode)?;
        let vector_dimensions = append_input.vector_plaintext.as_ref().map_or(0, Vec::len);
        let owner_records = self.records_by_owner.entry(append_input.owner_did_key.clone()).or_default();
        validate_owner_dimensions(owner_records, vector_dimensions)?;

        let sequence = owner_records.len() as u64 + 1;
        let hash_chain_prev = previous_hash(owner_records);
        let record = build_record(
            append_input,
            vector_dimensions,
            sequence,
            hash_chain_prev,
        );
        owner_records.push(record.clone());
        self.seen_embedding_ids.insert(record.embedding_id.clone());
        Ok(record)
    }
}

struct PreparedAppendInput {
    embedding_id: String,
    message_id: String,
    owner_did_key: String,
    agent_did: String,
    retention_class: ContentRetentionClass,
    model_id: String,
    vector_encrypted: Vec<u8>,
    vector_plaintext: Option<Vec<f32>>,
    created_at_epoch_seconds: u64,
    privacy_mode: DataLayerM5EmbeddingPrivacyMode,
}

fn prepare_append_input(
    input: DataLayerM5EmbeddingRecordInput,
    seen_embedding_ids: &std::collections::BTreeSet<String>,
    privacy_mode: DataLayerM5EmbeddingPrivacyMode,
) -> Result<PreparedAppendInput, DataLayerM5VectorIntegrationError> {
    validate_ingest_input(&input)?;
    let parsed_owner_did = parse_kamn_did(input.owner_did.as_str())?;
    let parsed_agent_did = validate_agent_did(input.agent_did.as_str())?;
    reject_duplicate_embedding(seen_embedding_ids, input.embedding_id.as_str())?;
    Ok(PreparedAppendInput {
        embedding_id: input.embedding_id,
        message_id: input.message_id,
        owner_did_key: parsed_owner_did.as_str().to_owned(),
        agent_did: parsed_agent_did.as_str().to_owned(),
        retention_class: input.retention_class,
        model_id: input.model_id,
        vector_encrypted: input.vector_encrypted,
        vector_plaintext: resolve_plaintext_vector(privacy_mode, input.vector_plaintext)?,
        created_at_epoch_seconds: input.created_at_epoch_seconds,
        privacy_mode,
    })
}

fn validate_ingest_input(
    input: &DataLayerM5EmbeddingRecordInput,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    validate_non_empty(input.embedding_id.as_str(), "embedding_id")?;
    validate_non_empty(input.message_id.as_str(), "message_id")?;
    validate_non_empty(input.model_id.as_str(), "model_id")?;
    if input.vector_encrypted.is_empty() {
        return Err(DataLayerM5VectorIntegrationError::EmptyField("vector_encrypted"));
    }
    if input.created_at_epoch_seconds == 0 {
        return Err(DataLayerM5VectorIntegrationError::EmptyField(
            "created_at_epoch_seconds",
        ));
    }
    Ok(())
}

fn reject_duplicate_embedding(
    seen_embedding_ids: &std::collections::BTreeSet<String>,
    embedding_id: &str,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    if seen_embedding_ids.contains(embedding_id) {
        return Err(DataLayerM5VectorIntegrationError::DuplicateEmbeddingId(
            embedding_id.to_owned(),
        ));
    }
    Ok(())
}

fn resolve_plaintext_vector(
    privacy_mode: DataLayerM5EmbeddingPrivacyMode,
    vector_plaintext: Option<Vec<f32>>,
) -> Result<Option<Vec<f32>>, DataLayerM5VectorIntegrationError> {
    match (privacy_mode, vector_plaintext) {
        (DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted, Some(_)) => Err(
            DataLayerM5VectorIntegrationError::PrivacyModeViolation {
                reason_code: DATA_LAYER_M5_OWNER_SIDE_PLAINTEXT_STORAGE_NOT_ALLOWED_REASON_CODE,
            },
        ),
        (DataLayerM5EmbeddingPrivacyMode::OwnerSideEncrypted, None) => Ok(None),
        (DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn, None) => Err(
            DataLayerM5VectorIntegrationError::PrivacyModeViolation {
                reason_code: DATA_LAYER_M5_SERVER_SIDE_PLAINTEXT_REQUIRED_REASON_CODE,
            },
        ),
        (DataLayerM5EmbeddingPrivacyMode::ServerSidePlaintextOptIn, Some(vector)) => {
            Ok(Some(validate_vector(vector, "vector_plaintext")?))
        }
    }
}

fn validate_owner_dimensions(
    owner_records: &[DataLayerM5EmbeddingRecord],
    vector_dimensions: usize,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    if let Some(expected_dimensions) = owner_vector_dimensions(owner_records) {
        if vector_dimensions > 0 && vector_dimensions != expected_dimensions {
            return Err(DataLayerM5VectorIntegrationError::InvalidVectorDimensions {
                expected: expected_dimensions,
                found: vector_dimensions,
            });
        }
    }
    Ok(())
}

fn previous_hash(owner_records: &[DataLayerM5EmbeddingRecord]) -> String {
    owner_records
        .last()
        .map(|record| record.record_hash.clone())
        .unwrap_or_else(|| DATA_LAYER_M5_EMBEDDING_HASH_CHAIN_GENESIS.to_owned())
}

fn build_record(
    append_input: PreparedAppendInput,
    vector_dimensions: usize,
    sequence: u64,
    hash_chain_prev: String,
) -> DataLayerM5EmbeddingRecord {
    let material = DataLayerM5RecordHashMaterial {
        embedding_id: append_input.embedding_id.as_str(),
        message_id: append_input.message_id.as_str(),
        owner_did: append_input.owner_did_key.as_str(),
        agent_did: append_input.agent_did.as_str(),
        retention_class: append_input.retention_class,
        model_id: append_input.model_id.as_str(),
        vector_encrypted: append_input.vector_encrypted.as_slice(),
        vector_plaintext: append_input.vector_plaintext.as_deref(),
        vector_dimensions,
        created_at_epoch_seconds: append_input.created_at_epoch_seconds,
        privacy_mode: append_input.privacy_mode,
    };
    let record_hash = compute_embedding_record_hash(sequence, &material, hash_chain_prev.as_str());
    record_from_append_input(append_input, vector_dimensions, sequence, hash_chain_prev, record_hash)
}

fn record_from_append_input(
    append_input: PreparedAppendInput,
    vector_dimensions: usize,
    sequence: u64,
    hash_chain_prev: String,
    record_hash: String,
) -> DataLayerM5EmbeddingRecord {
    DataLayerM5EmbeddingRecord {
        embedding_id: append_input.embedding_id,
        message_id: append_input.message_id,
        owner_did: append_input.owner_did_key,
        agent_did: append_input.agent_did,
        retention_class: append_input.retention_class,
        model_id: append_input.model_id,
        vector_encrypted: append_input.vector_encrypted,
        privacy_mode: append_input.privacy_mode,
        vector_plaintext: append_input.vector_plaintext,
        vector_dimensions,
        sequence,
        created_at_epoch_seconds: append_input.created_at_epoch_seconds,
        hash_chain_prev,
        record_hash,
    }
}
