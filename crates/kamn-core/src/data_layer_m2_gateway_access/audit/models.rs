/// Input envelope for one access-audit append event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2AccessAuditInput {
    /// Requester did carried by this public contract model.
    pub requester_did: String,
    /// Action carried by this public contract model.
    pub action: String,
    /// Resource id carried by this public contract model.
    pub resource_id: String,
    /// Reason code carried by this public contract model.
    pub reason_code: String,
    /// Event epoch seconds carried by this public contract model.
    pub event_epoch_seconds: u64,
}

/// Stored append-only access-audit record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2AccessAuditRecord {
    /// Sequence carried by this public contract model.
    pub sequence: u64,
    /// Requester did carried by this public contract model.
    pub requester_did: String,
    /// Action carried by this public contract model.
    pub action: String,
    /// Resource id carried by this public contract model.
    pub resource_id: String,
    /// Reason code carried by this public contract model.
    pub reason_code: String,
    /// Event epoch seconds carried by this public contract model.
    pub event_epoch_seconds: u64,
    /// Hash chain prev carried by this public contract model.
    pub hash_chain_prev: String,
    /// Record hash carried by this public contract model.
    pub record_hash: String,
}
