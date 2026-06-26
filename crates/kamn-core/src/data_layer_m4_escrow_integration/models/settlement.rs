use super::DataLayerM4EscrowState;
use std::collections::BTreeMap;
use std::fmt;

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

/// Error taxonomy for M4 escrow transition/visibility/evidence contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM4SettlementEvidenceRegistryError {
    /// Empty field variant for this public contract enum.
    EmptyField(&'static str),
    /// Invalid did variant for this public contract enum.
    InvalidDid(String),
    /// Invalid escrow parties variant for this public contract enum.
    InvalidEscrowParties(&'static str),
    /// Duplicate escrow id variant for this public contract enum.
    DuplicateEscrowId(String),
    /// Escrow not found variant for this public contract enum.
    EscrowNotFound {
        /// String carried by this public contract model.
        escrow_id: String,
    },
    /// Invalid escrow transition variant for this public contract enum.
    InvalidEscrowTransition {
        /// String carried by this public contract model.
        escrow_id: String,
        /// Data layer m4 escrow state carried by this public contract model.
        from: DataLayerM4EscrowState,
        /// Str carried by this public contract model.
        action: &'static str,
    },
    /// Invalid auditor threshold variant for this public contract enum.
    InvalidAuditorThreshold {
        /// U8 carried by this public contract model.
        threshold: u8,
        /// Usize carried by this public contract model.
        share_holder_count: usize,
    },
    /// Invalid hash token variant for this public contract enum.
    InvalidHashToken(&'static str),
    /// Unsupported settlement state variant for this public contract enum.
    UnsupportedSettlementState(DataLayerM4EscrowState),
    /// Invalid evidence hash chain variant for this public contract enum.
    InvalidEvidenceHashChain {
        /// String carried by this public contract model.
        escrow_id: String,
        /// Usize carried by this public contract model.
        position: usize,
        /// Str carried by this public contract model.
        reason: &'static str,
    },
    /// Evidence sequence not found variant for this public contract enum.
    EvidenceSequenceNotFound {
        /// String carried by this public contract model.
        escrow_id: String,
        /// U64 carried by this public contract model.
        sequence: u64,
    },
}

impl fmt::Display for DataLayerM4SettlementEvidenceRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if write_structured_error(self, f)? {
            return Ok(());
        }
        write_simple_error(self, f)
    }
}

impl std::error::Error for DataLayerM4SettlementEvidenceRegistryError {}

fn write_structured_error(
    error: &DataLayerM4SettlementEvidenceRegistryError,
    f: &mut fmt::Formatter<'_>,
) -> Result<bool, fmt::Error> {
    match error {
        DataLayerM4SettlementEvidenceRegistryError::InvalidEscrowTransition {
            escrow_id,
            from,
            action,
        } => write_invalid_escrow_transition(f, escrow_id, *from, action).map(|_| true),
        DataLayerM4SettlementEvidenceRegistryError::InvalidAuditorThreshold {
            threshold,
            share_holder_count,
        } => write_invalid_auditor_threshold(f, *threshold, *share_holder_count).map(|_| true),
        DataLayerM4SettlementEvidenceRegistryError::InvalidEvidenceHashChain {
            escrow_id,
            position,
            reason,
        } => write_invalid_evidence_hash_chain(f, escrow_id, *position, reason).map(|_| true),
        _ => Ok(false),
    }
}

fn write_simple_error(
    error: &DataLayerM4SettlementEvidenceRegistryError,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    if write_identity_error(error, f)? {
        return Ok(());
    }
    write_state_error(error, f)
}

fn write_identity_error(
    error: &DataLayerM4SettlementEvidenceRegistryError,
    f: &mut fmt::Formatter<'_>,
) -> Result<bool, fmt::Error> {
    match error {
        DataLayerM4SettlementEvidenceRegistryError::EmptyField(field) => {
            write!(f, "{field} must not be empty").map(|_| true)
        }
        DataLayerM4SettlementEvidenceRegistryError::InvalidDid(value) => {
            write!(f, "invalid did: {value}").map(|_| true)
        }
        DataLayerM4SettlementEvidenceRegistryError::InvalidEscrowParties(reason) => {
            write!(f, "invalid escrow parties: {reason}").map(|_| true)
        }
        DataLayerM4SettlementEvidenceRegistryError::DuplicateEscrowId(escrow_id) => {
            write!(f, "duplicate escrow_id: {escrow_id}").map(|_| true)
        }
        DataLayerM4SettlementEvidenceRegistryError::EscrowNotFound { escrow_id } => {
            write!(f, "escrow not found: {escrow_id}").map(|_| true)
        }
        DataLayerM4SettlementEvidenceRegistryError::InvalidHashToken(field) => {
            write!(f, "invalid hash token: {field}").map(|_| true)
        }
        _ => Ok(false),
    }
}

fn write_state_error(
    error: &DataLayerM4SettlementEvidenceRegistryError,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    match error {
        DataLayerM4SettlementEvidenceRegistryError::UnsupportedSettlementState(state) => {
            write!(f, "unsupported settlement state: {:?}", state)
        }
        DataLayerM4SettlementEvidenceRegistryError::EvidenceSequenceNotFound {
            escrow_id,
            sequence,
        } => write!(f, "evidence sequence not found for {escrow_id}: {sequence}"),
        _ => unreachable!("structured settlement error variants handled first"),
    }
}

fn write_invalid_escrow_transition(
    f: &mut fmt::Formatter<'_>,
    escrow_id: &str,
    from: DataLayerM4EscrowState,
    action: &str,
) -> fmt::Result {
    write!(
        f,
        "invalid escrow transition for {escrow_id}: from {:?} via {action}",
        from
    )
}

fn write_invalid_auditor_threshold(
    f: &mut fmt::Formatter<'_>,
    threshold: u8,
    share_holder_count: usize,
) -> fmt::Result {
    write!(
        f,
        "invalid auditor threshold {threshold} for {share_holder_count} share holders"
    )
}

fn write_invalid_evidence_hash_chain(
    f: &mut fmt::Formatter<'_>,
    escrow_id: &str,
    position: usize,
    reason: &str,
) -> fmt::Result {
    write!(
        f,
        "invalid evidence hash chain for {escrow_id} at position {position}: {reason}"
    )
}
