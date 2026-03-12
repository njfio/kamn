use super::super::models::*;
use super::super::support::*;
use crate::ContentRetentionClass;

impl DataLayerM5EmbeddingRegistry {
    /// Appends one embedding record under owner scope.
    pub fn append(
        &mut self,
        input: DataLayerM5EmbeddingRecordInput,
    ) -> Result<DataLayerM5EmbeddingRecord, DataLayerM5VectorIntegrationError> {
        validate_ingest_input(&input)?;
        let DataLayerM5EmbeddingRecordInput {
            embedding_id,
            message_id,
            owner_did,
            agent_did,
            retention_class,
            model_id,
            vector_encrypted,
            vector_plaintext,
            created_at_epoch_seconds,
        } = input;
        let parsed_owner_did = parse_kamn_did(owner_did.as_str())?;
        let owner_did_key = parsed_owner_did.as_str().to_owned();
        let parsed_agent_did = validate_agent_did(agent_did.as_str())?;
        reject_duplicate_embedding(&self.seen_embedding_ids, embedding_id.as_str())?;

        let vector_plaintext = resolve_plaintext_vector(self.privacy_mode, vector_plaintext)?;
        let vector_dimensions = vector_plaintext.as_ref().map_or(0, Vec::len);
        let owner_records = self.records_by_owner.entry(owner_did_key.clone()).or_default();
        validate_owner_dimensions(owner_records, vector_dimensions)?;

        let sequence = owner_records.len() as u64 + 1;
        let hash_chain_prev = previous_hash(owner_records);
        let record = build_record(
            embedding_id,
            message_id,
            &owner_did_key,
            parsed_agent_did.as_str(),
            retention_class,
            model_id,
            vector_encrypted,
            vector_plaintext,
            vector_dimensions,
            self.privacy_mode,
            sequence,
            created_at_epoch_seconds,
            hash_chain_prev,
        );
        owner_records.push(record.clone());
        self.seen_embedding_ids.insert(record.embedding_id.clone());
        Ok(record)
    }
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
    embedding_id: String,
    message_id: String,
    owner_did_key: &str,
    agent_did: &str,
    retention_class: ContentRetentionClass,
    model_id: String,
    vector_encrypted: Vec<u8>,
    vector_plaintext: Option<Vec<f32>>,
    vector_dimensions: usize,
    privacy_mode: DataLayerM5EmbeddingPrivacyMode,
    sequence: u64,
    created_at_epoch_seconds: u64,
    hash_chain_prev: String,
) -> DataLayerM5EmbeddingRecord {
    let material = DataLayerM5RecordHashMaterial {
        embedding_id: embedding_id.as_str(),
        message_id: message_id.as_str(),
        owner_did: owner_did_key,
        agent_did,
        retention_class,
        model_id: model_id.as_str(),
        vector_encrypted: vector_encrypted.as_slice(),
        vector_plaintext: vector_plaintext.as_deref(),
        vector_dimensions,
        created_at_epoch_seconds,
        privacy_mode,
    };
    let record_hash = compute_embedding_record_hash(sequence, &material, hash_chain_prev.as_str());
    DataLayerM5EmbeddingRecord {
        embedding_id,
        message_id,
        owner_did: owner_did_key.to_owned(),
        agent_did: agent_did.to_owned(),
        retention_class,
        model_id,
        vector_encrypted,
        privacy_mode,
        vector_plaintext,
        vector_dimensions,
        sequence,
        created_at_epoch_seconds,
        hash_chain_prev,
        record_hash,
    }
}
