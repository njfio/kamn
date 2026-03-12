/// Input envelope for one access-audit append event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2AccessAuditInput {
    pub requester_did: String,
    pub action: String,
    pub resource_id: String,
    pub reason_code: String,
    pub event_epoch_seconds: u64,
}

/// Stored append-only access-audit record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2AccessAuditRecord {
    pub sequence: u64,
    pub requester_did: String,
    pub action: String,
    pub resource_id: String,
    pub reason_code: String,
    pub event_epoch_seconds: u64,
    pub hash_chain_prev: String,
    pub record_hash: String,
}
