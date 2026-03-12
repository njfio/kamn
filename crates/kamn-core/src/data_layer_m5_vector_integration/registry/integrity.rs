use super::super::models::*;
use super::super::support::{compute_embedding_record_hash, parse_kamn_did, DataLayerM5RecordHashMaterial, validate_non_empty};

impl DataLayerM5EmbeddingRegistry {
    /// Verifies hash-chain integrity for one owner-scoped embedding stream.
    pub fn verify_owner_integrity(
        &self,
        owner_did: &str,
    ) -> Result<(), DataLayerM5VectorIntegrationError> {
        let owner_did = parse_kamn_did(owner_did)?;
        let owner_did_key = owner_did.as_str();
        let records = self.records_by_owner.get(owner_did_key).ok_or_else(|| {
            DataLayerM5VectorIntegrationError::OwnerNotFound {
                owner_did: owner_did_key.to_owned(),
            }
        })?;
        let mut expected_prev = DATA_LAYER_M5_EMBEDDING_HASH_CHAIN_GENESIS.to_owned();
        for (position, record) in records.iter().enumerate() {
            verify_record_link(record, owner_did_key, position, expected_prev.as_str())?;
            expected_prev = record.record_hash.clone();
        }
        Ok(())
    }

    /// Replaces one record hash without recomputing chain links.
    pub fn replace_record_hash_unchecked(
        &mut self,
        owner_did: &str,
        sequence: u64,
        record_hash: &str,
    ) -> Result<(), DataLayerM5VectorIntegrationError> {
        let owner_did = parse_kamn_did(owner_did)?;
        let owner_did_key = owner_did.as_str();
        validate_non_empty(record_hash, "record_hash")?;
        let records = self.records_by_owner.get_mut(owner_did_key).ok_or_else(|| {
            DataLayerM5VectorIntegrationError::OwnerNotFound {
                owner_did: owner_did_key.to_owned(),
            }
        })?;
        let record = records.iter_mut().find(|entry| entry.sequence == sequence).ok_or_else(|| {
            DataLayerM5VectorIntegrationError::EmbeddingSequenceNotFound {
                owner_did: owner_did_key.to_owned(),
                sequence,
            }
        })?;
        record.record_hash = record_hash.to_owned();
        Ok(())
    }
}

fn verify_record_link(
    record: &DataLayerM5EmbeddingRecord,
    owner_did_key: &str,
    position: usize,
    expected_prev: &str,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    if record.hash_chain_prev != expected_prev {
        return invalid_hash_chain(owner_did_key, position, "hash_chain_prev mismatch");
    }
    let hash_material = record_hash_material(record);
    let expected_hash =
        compute_embedding_record_hash(record.sequence, &hash_material, record.hash_chain_prev.as_str());
    if record.record_hash != expected_hash {
        return invalid_hash_chain(owner_did_key, position, "record_hash mismatch");
    }
    Ok(())
}

fn record_hash_material(record: &DataLayerM5EmbeddingRecord) -> DataLayerM5RecordHashMaterial<'_> {
    DataLayerM5RecordHashMaterial {
        embedding_id: record.embedding_id.as_str(),
        message_id: record.message_id.as_str(),
        owner_did: record.owner_did.as_str(),
        agent_did: record.agent_did.as_str(),
        retention_class: record.retention_class,
        model_id: record.model_id.as_str(),
        vector_encrypted: record.vector_encrypted.as_slice(),
        vector_plaintext: record.vector_plaintext.as_deref(),
        vector_dimensions: record.vector_dimensions,
        created_at_epoch_seconds: record.created_at_epoch_seconds,
        privacy_mode: record.privacy_mode,
    }
}

fn invalid_hash_chain(
    owner_did: &str,
    position: usize,
    reason: &'static str,
) -> Result<(), DataLayerM5VectorIntegrationError> {
    Err(DataLayerM5VectorIntegrationError::InvalidEmbeddingHashChain {
        owner_did: owner_did.to_owned(),
        position,
        reason,
    })
}
