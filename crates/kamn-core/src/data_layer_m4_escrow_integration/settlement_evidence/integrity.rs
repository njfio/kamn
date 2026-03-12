use super::super::models::*;
use super::super::validation::{compute_evidence_hash, validate_non_empty};

impl DataLayerM4SettlementEvidenceRegistry {
    /// Verifies evidence hash-chain integrity for one escrow.
    pub fn verify_escrow_integrity(
        &self,
        escrow_id: &str,
    ) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
        validate_non_empty(escrow_id, "escrow_id")?;
        let records = self.records_by_escrow.get(escrow_id).ok_or_else(|| {
            DataLayerM4SettlementEvidenceRegistryError::EscrowNotFound {
                escrow_id: escrow_id.to_owned(),
            }
        })?;
        let mut expected_prev = DATA_LAYER_M4_EVIDENCE_HASH_CHAIN_GENESIS.to_owned();
        for (position, record) in records.iter().enumerate() {
            verify_record_prev_link(escrow_id, position, record, expected_prev.as_str())?;
            verify_record_hash(escrow_id, position, record)?;
            expected_prev = record.record_hash.clone();
        }
        Ok(())
    }

    /// Replaces one record hash without recomputing chain links.
    ///
    /// This helper intentionally bypasses integrity checks for tamper regression tests.
    pub fn replace_record_hash_unchecked(
        &mut self,
        escrow_id: &str,
        sequence: u64,
        record_hash: &str,
    ) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
        validate_non_empty(escrow_id, "escrow_id")?;
        validate_non_empty(record_hash, "record_hash")?;
        let records = self.records_by_escrow.get_mut(escrow_id).ok_or_else(|| {
            DataLayerM4SettlementEvidenceRegistryError::EscrowNotFound {
                escrow_id: escrow_id.to_owned(),
            }
        })?;
        let record = records.iter_mut().find(|entry| entry.sequence == sequence).ok_or_else(|| {
            DataLayerM4SettlementEvidenceRegistryError::EvidenceSequenceNotFound {
                escrow_id: escrow_id.to_owned(),
                sequence,
            }
        })?;
        record.record_hash = record_hash.to_owned();
        Ok(())
    }
}

fn verify_record_prev_link(
    escrow_id: &str,
    position: usize,
    record: &DataLayerM4SettlementEvidenceRecord,
    expected_prev: &str,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    if record.hash_chain_prev != expected_prev {
        return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidEvidenceHashChain {
            escrow_id: escrow_id.to_owned(),
            position,
            reason: "hash_chain_prev mismatch",
        });
    }
    Ok(())
}

fn verify_record_hash(
    escrow_id: &str,
    position: usize,
    record: &DataLayerM4SettlementEvidenceRecord,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    let expected_hash = compute_evidence_hash(
        record.sequence,
        &DataLayerM4SettlementEvidenceInput {
            escrow_id: record.escrow_id.clone(),
            escrow_state: record.escrow_state,
            settlement_receipt_hash: record.settlement_receipt_hash.clone(),
            settlement_payload_hash: record.settlement_payload_hash.clone(),
            recorded_at_epoch_seconds: record.recorded_at_epoch_seconds,
        },
        record.hash_chain_prev.as_str(),
    );
    if record.record_hash != expected_hash {
        return Err(DataLayerM4SettlementEvidenceRegistryError::InvalidEvidenceHashChain {
            escrow_id: escrow_id.to_owned(),
            position,
            reason: "record_hash mismatch",
        });
    }
    Ok(())
}
