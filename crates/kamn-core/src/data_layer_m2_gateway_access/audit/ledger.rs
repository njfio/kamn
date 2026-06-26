use super::models::{DataLayerM2AccessAuditInput, DataLayerM2AccessAuditRecord};
use crate::data_layer_m2_gateway_access::models::{
    compute_audit_record_hash, validate_audit_input, DataLayerM2GatewayError,
};
use crate::data_layer_m2_gateway_access::DATA_LAYER_M2_AUDIT_HASH_CHAIN_GENESIS;

/// Append-only access-audit ledger for M2 access-gateway decisions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM2AccessAuditLedger {
    records: Vec<DataLayerM2AccessAuditRecord>,
}

impl DataLayerM2AccessAuditLedger {
    /// Creates a new value for this public contract type.
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs the append contract operation.
    pub fn append(
        &mut self,
        input: DataLayerM2AccessAuditInput,
    ) -> Result<DataLayerM2AccessAuditRecord, DataLayerM2GatewayError> {
        validate_audit_input(&input)?;
        let record = build_record(self.records.last(), input);
        self.records.push(record.clone());
        Ok(record)
    }

    /// Runs the records contract operation.
    pub fn records(&self) -> &[DataLayerM2AccessAuditRecord] {
        &self.records
    }

    /// Runs the verify hash chain contract operation.
    pub fn verify_hash_chain(&self) -> Result<(), DataLayerM2GatewayError> {
        let mut expected_prev = DATA_LAYER_M2_AUDIT_HASH_CHAIN_GENESIS.to_owned();
        for (position, record) in self.records.iter().enumerate() {
            verify_prev_hash(position, record, expected_prev.as_str())?;
            verify_record_hash(position, record)?;
            expected_prev = record.record_hash.clone();
        }
        Ok(())
    }

    /// Runs the replace record hash unchecked contract operation.
    pub fn replace_record_hash_unchecked(
        &mut self,
        sequence: u64,
        record_hash: &str,
    ) -> Result<(), DataLayerM2GatewayError> {
        if record_hash.trim().is_empty() {
            return Err(DataLayerM2GatewayError::EmptyField("record_hash"));
        }
        let record = self
            .records
            .iter_mut()
            .find(|entry| entry.sequence == sequence)
            .ok_or(DataLayerM2GatewayError::AuditSequenceNotFound(sequence))?;
        record.record_hash = record_hash.to_owned();
        Ok(())
    }
}

fn build_record(
    previous: Option<&DataLayerM2AccessAuditRecord>,
    input: DataLayerM2AccessAuditInput,
) -> DataLayerM2AccessAuditRecord {
    let sequence = previous.map_or(1, |record| record.sequence + 1);
    let hash_chain_prev = previous
        .map(|record| record.record_hash.clone())
        .unwrap_or_else(|| DATA_LAYER_M2_AUDIT_HASH_CHAIN_GENESIS.to_owned());
    let record_hash = compute_audit_record_hash(sequence, &input, hash_chain_prev.as_str());
    DataLayerM2AccessAuditRecord {
        sequence,
        requester_did: input.requester_did,
        action: input.action,
        resource_id: input.resource_id,
        reason_code: input.reason_code,
        event_epoch_seconds: input.event_epoch_seconds,
        hash_chain_prev,
        record_hash,
    }
}

fn verify_prev_hash(
    position: usize,
    record: &DataLayerM2AccessAuditRecord,
    expected_prev: &str,
) -> Result<(), DataLayerM2GatewayError> {
    if record.hash_chain_prev == expected_prev {
        return Ok(());
    }
    Err(DataLayerM2GatewayError::InvalidAuditHashChain {
        position,
        reason: "hash_chain_prev mismatch",
    })
}

fn verify_record_hash(
    position: usize,
    record: &DataLayerM2AccessAuditRecord,
) -> Result<(), DataLayerM2GatewayError> {
    let expected_hash = compute_audit_record_hash(
        record.sequence,
        &audit_input(record),
        record.hash_chain_prev.as_str(),
    );
    if record.record_hash == expected_hash {
        return Ok(());
    }
    Err(DataLayerM2GatewayError::InvalidAuditHashChain {
        position,
        reason: "record_hash mismatch",
    })
}

fn audit_input(record: &DataLayerM2AccessAuditRecord) -> DataLayerM2AccessAuditInput {
    DataLayerM2AccessAuditInput {
        requester_did: record.requester_did.clone(),
        action: record.action.clone(),
        resource_id: record.resource_id.clone(),
        reason_code: record.reason_code.clone(),
        event_epoch_seconds: record.event_epoch_seconds,
    }
}
