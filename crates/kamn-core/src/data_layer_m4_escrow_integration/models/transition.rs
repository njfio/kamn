use super::DataLayerM4EscrowState;
use std::collections::BTreeMap;

/// Input for creating one escrow draft record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4EscrowDraftInput {
    /// Stable escrow identifier.
    pub escrow_id: String,
    /// Initiator DID.
    pub initiator_did: String,
    /// Counterparty DID.
    pub counterparty_did: String,
    /// Optional escrow auditor DID.
    pub auditor_did: Option<String>,
    /// Optional threshold shares required for auditor reconstruction.
    pub auditor_threshold: Option<u8>,
    /// DIDs of share holders for auditor reconstruction.
    pub auditor_share_holders: Vec<String>,
    /// Optional expiration timestamp.
    pub expires_at_epoch_seconds: Option<u64>,
}

/// Stored escrow projection managed by the M4 transition engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4EscrowRecord {
    /// Stable escrow identifier.
    pub escrow_id: String,
    /// Initiator DID.
    pub initiator_did: String,
    /// Counterparty DID.
    pub counterparty_did: String,
    /// Optional escrow auditor DID.
    pub auditor_did: Option<String>,
    /// Optional threshold shares required for auditor reconstruction.
    pub auditor_threshold: Option<u8>,
    /// DIDs of share holders for auditor reconstruction.
    pub auditor_share_holders: Vec<String>,
    /// Current escrow state.
    pub state: DataLayerM4EscrowState,
    /// Optional expiration timestamp.
    pub expires_at_epoch_seconds: Option<u64>,
    /// Optional dispute-opened timestamp.
    pub dispute_opened_at_epoch_seconds: Option<u64>,
    /// Optional settlement timestamp.
    pub settled_at_epoch_seconds: Option<u64>,
    /// Optional settlement receipt hash for final states.
    pub settlement_receipt_hash: Option<String>,
}

/// State transition action for one escrow record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM4EscrowTransitionAction {
    /// Move `Created -> Funded`.
    Fund { funded_at_epoch_seconds: u64 },
    /// Move `Funded -> Active`.
    Activate { activated_at_epoch_seconds: u64 },
    /// Move `Active -> Disputed`.
    OpenDispute {
        dispute_opened_at_epoch_seconds: u64,
    },
    /// Move `Active|Disputed -> Released`.
    ResolveRelease {
        settled_at_epoch_seconds: u64,
        settlement_receipt_hash: String,
    },
    /// Move `Active|Disputed -> Refunded`.
    ResolveRefund {
        settled_at_epoch_seconds: u64,
        settlement_receipt_hash: String,
    },
    /// Move `Created|Funded|Active -> Expired`.
    Expire { expired_at_epoch_seconds: u64 },
}

impl DataLayerM4EscrowTransitionAction {
    pub(crate) fn marker(&self) -> &'static str {
        match self {
            Self::Fund { .. } => "fund",
            Self::Activate { .. } => "activate",
            Self::OpenDispute { .. } => "open_dispute",
            Self::ResolveRelease { .. } => "resolve_release",
            Self::ResolveRefund { .. } => "resolve_refund",
            Self::Expire { .. } => "expire",
        }
    }
}

/// Transition evidence projected for successful escrow state mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM4EscrowTransitionEvidence {
    /// Escrow identifier.
    pub escrow_id: String,
    /// Previous state.
    pub from: DataLayerM4EscrowState,
    /// Action applied.
    pub action: DataLayerM4EscrowTransitionAction,
    /// Resulting state.
    pub to: DataLayerM4EscrowState,
    /// Stable reason code marker.
    pub reason_code: &'static str,
}

/// Transition and visibility engine for M4 escrow records.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM4EscrowTransitionEngine {
    pub(crate) escrows: BTreeMap<String, DataLayerM4EscrowRecord>,
}
