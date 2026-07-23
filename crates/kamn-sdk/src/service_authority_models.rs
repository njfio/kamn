/// Parsed response for task lifecycle mutations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceTaskTransitionReceipt {
    /// Task identifier.
    pub task_id: String,
    /// Resulting lifecycle state.
    pub state: String,
    /// Service-issued durable receipt identifier.
    pub receipt_id: String,
    /// Digest of the durable service receipt.
    pub receipt_digest: String,
    /// Canonical durable receipt action.
    pub action: String,
}

/// Durable settlement receipt returned with a finalized escrow release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSettlementReceipt {
    /// Settlement intent receipt identifier.
    pub receipt_id: String,
    /// Digest of the durable settlement intent.
    pub receipt_digest: String,
    /// Canonical settlement receipt action.
    pub action: String,
    /// Escrow identifier bound by the settlement receipt.
    pub resource_id: String,
    /// Durable settlement intent state.
    pub state: String,
}
