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
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Created at epoch seconds carried by this public contract model.
    pub created_at_epoch_seconds: u64,
    /// Content hash carried by this public contract model.
    pub content_hash: String,
    /// Hash chain prev carried by this public contract model.
    pub hash_chain_prev: String,
    /// Retention class carried by this public contract model.
    pub retention_class: DataLayerM8RetentionClass,
    /// Retention extension seconds carried by this public contract model.
    pub retention_extension_seconds: u64,
    /// Wrapped keys carried by this public contract model.
    pub wrapped_keys: Vec<DataLayerM8WrappedCekInput>,
}

/// Stored message lifecycle record for M8 compliance controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8MessageRecord {
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Created at epoch seconds carried by this public contract model.
    pub created_at_epoch_seconds: u64,
    /// Content hash carried by this public contract model.
    pub content_hash: String,
    /// Hash chain prev carried by this public contract model.
    pub hash_chain_prev: String,
    /// Retention class carried by this public contract model.
    pub retention_class: DataLayerM8RetentionClass,
    /// Retention extension seconds carried by this public contract model.
    pub retention_extension_seconds: u64,
    /// Wrapped keys carried by this public contract model.
    pub wrapped_keys: Vec<DataLayerM8WrappedCekInput>,
    /// Legal hold active carried by this public contract model.
    pub legal_hold_active: bool,
    /// Shredded at epoch seconds carried by this public contract model.
    pub shredded_at_epoch_seconds: Option<u64>,
    /// Shred reason code carried by this public contract model.
    pub shred_reason_code: Option<&'static str>,
    /// Sequence carried by this public contract model.
    pub sequence: u64,
}

/// Owner scope query for retention worker candidate projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8OwnerScopeQuery {
    /// Requester owner did carried by this public contract model.
    pub requester_owner_did: String,
    /// Owner did carried by this public contract model.
    pub owner_did: String,
}

/// Legal-hold mutation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8LegalHoldRequest {
    /// Requester owner did carried by this public contract model.
    pub requester_owner_did: String,
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Legal hold active carried by this public contract model.
    pub legal_hold_active: bool,
}

/// Crypto-shred mutation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8CryptoShredRequest {
    /// Requester owner did carried by this public contract model.
    pub requester_owner_did: String,
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Shredded at epoch seconds carried by this public contract model.
    pub shredded_at_epoch_seconds: u64,
}

/// Retention-due candidate projected for retention worker execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8RetentionDueCandidate {
    /// Owner did carried by this public contract model.
    pub owner_did: String,
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Retention class carried by this public contract model.
    pub retention_class: DataLayerM8RetentionClass,
    /// Due at epoch seconds carried by this public contract model.
    pub due_at_epoch_seconds: u64,
    /// Reason code carried by this public contract model.
    pub reason_code: &'static str,
}

/// M8 compliance registry for owner-scoped retention and shredding controls.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM8ComplianceRegistry {
    pub(crate) messages_by_owner: BTreeMap<String, Vec<DataLayerM8MessageRecord>>,
}
