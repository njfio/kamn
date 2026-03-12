use super::super::models::*;
use super::super::validation::{compute_evidence_hash, validate_hash_token, validate_non_empty, validate_non_zero_timestamp};

impl DataLayerM4SettlementEvidenceRegistry {
    /// Creates an empty settlement evidence registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one settlement evidence record.
    pub fn append(
        &mut self,
        input: DataLayerM4SettlementEvidenceInput,
    ) -> Result<DataLayerM4SettlementEvidenceRecord, DataLayerM4SettlementEvidenceRegistryError> {
        validate_append_input(&input)?;
        let escrow_records = self.records_by_escrow.entry(input.escrow_id.clone()).or_default();
        let sequence = (escrow_records.len() + 1) as u64;
        let hash_chain_prev = escrow_records
            .last()
            .map(|entry| entry.record_hash.clone())
            .unwrap_or_else(|| DATA_LAYER_M4_EVIDENCE_HASH_CHAIN_GENESIS.to_owned());
        let record_hash = compute_evidence_hash(sequence, &input, hash_chain_prev.as_str());
        let record = DataLayerM4SettlementEvidenceRecord {
            escrow_id: input.escrow_id,
            sequence,
            escrow_state: input.escrow_state,
            settlement_receipt_hash: input.settlement_receipt_hash,
            settlement_payload_hash: input.settlement_payload_hash,
            recorded_at_epoch_seconds: input.recorded_at_epoch_seconds,
            hash_chain_prev,
            record_hash,
        };
        escrow_records.push(record.clone());
        Ok(record)
    }
}

fn validate_append_input(
    input: &DataLayerM4SettlementEvidenceInput,
) -> Result<(), DataLayerM4SettlementEvidenceRegistryError> {
    validate_non_empty(input.escrow_id.as_str(), "escrow_id")?;
    validate_non_zero_timestamp(input.recorded_at_epoch_seconds, "recorded_at_epoch_seconds")?;
    validate_hash_token(input.settlement_receipt_hash.as_str(), "settlement_receipt_hash")?;
    validate_hash_token(input.settlement_payload_hash.as_str(), "settlement_payload_hash")?;
    if input.escrow_state != DataLayerM4EscrowState::Released
        && input.escrow_state != DataLayerM4EscrowState::Refunded
    {
        return Err(DataLayerM4SettlementEvidenceRegistryError::UnsupportedSettlementState(
            input.escrow_state,
        ));
    }
    Ok(())
}
