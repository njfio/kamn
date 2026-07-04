use super::DataLayerM4EscrowState;
use std::collections::BTreeMap;

/// Input envelope for one settlement evidence append event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4SettlementEvidenceInput {
    /// Escrow id carried by this public contract model.
    pub escrow_id: String,
    /// Escrow state carried by this public contract model.
    pub escrow_state: DataLayerM4EscrowState,
    /// Settlement receipt hash carried by this public contract model.
    pub settlement_receipt_hash: String,
    /// Settlement payload hash carried by this public contract model.
    pub settlement_payload_hash: String,
    /// Recorded at epoch seconds carried by this public contract model.
    pub recorded_at_epoch_seconds: u64,
}

/// Stored append-only settlement evidence record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4SettlementEvidenceRecord {
    /// Escrow id carried by this public contract model.
    pub escrow_id: String,
    /// Sequence carried by this public contract model.
    pub sequence: u64,
    /// Escrow state carried by this public contract model.
    pub escrow_state: DataLayerM4EscrowState,
    /// Settlement receipt hash carried by this public contract model.
    pub settlement_receipt_hash: String,
    /// Settlement payload hash carried by this public contract model.
    pub settlement_payload_hash: String,
    /// Recorded at epoch seconds carried by this public contract model.
    pub recorded_at_epoch_seconds: u64,
    /// Hash chain prev carried by this public contract model.
    pub hash_chain_prev: String,
    /// Record hash carried by this public contract model.
    pub record_hash: String,
}

/// Settlement evidence reconciliation decision for one terminal escrow projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM4SettlementEvidenceReconciliationDecision {
    /// Match variant for this public contract enum.
    Match,
    /// Mismatch variant for this public contract enum.
    Mismatch,
}

/// Reconciliation report linking terminal escrow projection to latest evidence row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4SettlementEvidenceReconciliationReport {
    /// Escrow id carried by this public contract model.
    pub escrow_id: String,
    /// Decision carried by this public contract model.
    pub decision: DataLayerM4SettlementEvidenceReconciliationDecision,
    /// Reason code carried by this public contract model.
    pub reason_code: &'static str,
    /// Escrow state carried by this public contract model.
    pub escrow_state: DataLayerM4EscrowState,
    /// Escrow settlement receipt hash carried by this public contract model.
    pub escrow_settlement_receipt_hash: String,
    /// Evidence sequence carried by this public contract model.
    pub evidence_sequence: Option<u64>,
    /// Evidence state carried by this public contract model.
    pub evidence_state: Option<DataLayerM4EscrowState>,
    /// Evidence settlement receipt hash carried by this public contract model.
    pub evidence_settlement_receipt_hash: Option<String>,
    /// Evidence settlement payload hash carried by this public contract model.
    pub evidence_settlement_payload_hash: Option<String>,
}

/// Append-only settlement evidence registry keyed by escrow identifier.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM4SettlementEvidenceRegistry {
    pub(crate) records_by_escrow: BTreeMap<String, Vec<DataLayerM4SettlementEvidenceRecord>>,
}
