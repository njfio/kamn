use std::collections::BTreeMap;

use super::DataLayerM8RetentionClass;

/// Wrapped CEK payload bound to one recipient DID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8WrappedCekInput {
    /// Recipient DID authorized for the wrapped CEK.
    pub recipient_did: String,
    /// Wrapped CEK bytes (encoded).
    pub wrapped_cek: String,
}

/// Input payload for registering one owner-scoped message lifecycle record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8MessageRecordInput {
    pub owner_did: String,
    pub message_id: String,
    pub created_at_epoch_seconds: u64,
    pub content_hash: String,
    pub hash_chain_prev: String,
    pub retention_class: DataLayerM8RetentionClass,
    pub retention_extension_seconds: u64,
    pub wrapped_keys: Vec<DataLayerM8WrappedCekInput>,
}

/// Stored message lifecycle record for M8 compliance controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8MessageRecord {
    pub owner_did: String,
    pub message_id: String,
    pub created_at_epoch_seconds: u64,
    pub content_hash: String,
    pub hash_chain_prev: String,
    pub retention_class: DataLayerM8RetentionClass,
    pub retention_extension_seconds: u64,
    pub wrapped_keys: Vec<DataLayerM8WrappedCekInput>,
    pub legal_hold_active: bool,
    pub shredded_at_epoch_seconds: Option<u64>,
    pub shred_reason_code: Option<&'static str>,
    pub sequence: u64,
}

/// Owner scope query for retention worker candidate projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8OwnerScopeQuery {
    pub requester_owner_did: String,
    pub owner_did: String,
}

/// Legal-hold mutation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8LegalHoldRequest {
    pub requester_owner_did: String,
    pub owner_did: String,
    pub message_id: String,
    pub legal_hold_active: bool,
}

/// Crypto-shred mutation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8CryptoShredRequest {
    pub requester_owner_did: String,
    pub owner_did: String,
    pub message_id: String,
    pub shredded_at_epoch_seconds: u64,
}

/// Retention-due candidate projected for retention worker execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8RetentionDueCandidate {
    pub owner_did: String,
    pub message_id: String,
    pub retention_class: DataLayerM8RetentionClass,
    pub due_at_epoch_seconds: u64,
    pub reason_code: &'static str,
}

/// M8 compliance registry for owner-scoped retention and shredding controls.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM8ComplianceRegistry {
    pub(crate) messages_by_owner: BTreeMap<String, Vec<DataLayerM8MessageRecord>>,
}
