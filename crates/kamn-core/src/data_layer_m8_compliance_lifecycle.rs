//! M8 compliance lifecycle contracts for crypto-shredding and retention controls.
//!
//! This module models PRD M8 behavior as deterministic Rust contracts:
//! owner-scoped message retention windows, legal-hold precedence, and
//! irreversible CEK shredding markers while preserving append-only integrity.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Ephemeral retention window (24 hours).
pub const DATA_LAYER_M8_EPHEMERAL_RETENTION_SECONDS: u64 = 86_400;
/// Standard retention window (90 days).
pub const DATA_LAYER_M8_STANDARD_RETENTION_SECONDS: u64 = 7_776_000;
/// Extended retention window (365 days).
pub const DATA_LAYER_M8_EXTENDED_RETENTION_SECONDS: u64 = 31_536_000;
/// Stable wrapped-CEK tombstone marker for crypto-shredded messages.
pub const DATA_LAYER_M8_CEK_TOMBSTONE_MARKER: &str = "m8:cek:crypto-shredded";
/// Stable reason marker for owner-scope authorization failures.
pub const DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE: &str = "m8_compliance_owner_scope_denied";
/// Stable reason marker for retention-due projections.
pub const DATA_LAYER_M8_RETENTION_DUE_REASON_CODE: &str = "m8_compliance_retention_due";
/// Stable reason marker for successful crypto-shredding transitions.
pub const DATA_LAYER_M8_CRYPTO_SHRED_REASON_CODE: &str = "m8_compliance_crypto_shred_applied";

/// Retention class contract for M8 compliance lifecycle controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataLayerM8RetentionClass {
    /// Default profile: 90 days.
    Standard,
    /// Compliance-sensitive profile: 1 year.
    Extended,
    /// Hold profile: shredding blocked until explicit release.
    LegalHold,
    /// Never shred automatically.
    Permanent,
    /// Short-lived profile: 24 hours.
    Ephemeral,
}

impl DataLayerM8RetentionClass {
    fn retention_window_seconds(self) -> Option<u64> {
        match self {
            Self::Standard => Some(DATA_LAYER_M8_STANDARD_RETENTION_SECONDS),
            Self::Extended => Some(DATA_LAYER_M8_EXTENDED_RETENTION_SECONDS),
            Self::LegalHold => None,
            Self::Permanent => None,
            Self::Ephemeral => Some(DATA_LAYER_M8_EPHEMERAL_RETENTION_SECONDS),
        }
    }
}

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
    /// Owner DID scope.
    pub owner_did: String,
    /// Stable message identifier.
    pub message_id: String,
    /// Message creation timestamp in epoch seconds.
    pub created_at_epoch_seconds: u64,
    /// Content hash marker preserved across shredding.
    pub content_hash: String,
    /// Hash-chain previous marker preserved across shredding.
    pub hash_chain_prev: String,
    /// Retention class policy for this message.
    pub retention_class: DataLayerM8RetentionClass,
    /// Additional retention extension seconds (escrow/dispute/regulatory).
    pub retention_extension_seconds: u64,
    /// Wrapped CEK entries for authorized recipients.
    pub wrapped_keys: Vec<DataLayerM8WrappedCekInput>,
}

/// Stored message lifecycle record for M8 compliance controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8MessageRecord {
    /// Owner DID scope.
    pub owner_did: String,
    /// Stable message identifier.
    pub message_id: String,
    /// Message creation timestamp in epoch seconds.
    pub created_at_epoch_seconds: u64,
    /// Content hash marker preserved across shredding.
    pub content_hash: String,
    /// Hash-chain previous marker preserved across shredding.
    pub hash_chain_prev: String,
    /// Retention class policy for this message.
    pub retention_class: DataLayerM8RetentionClass,
    /// Additional retention extension seconds (escrow/dispute/regulatory).
    pub retention_extension_seconds: u64,
    /// Wrapped CEK entries (replaced with tombstone marker when shredded).
    pub wrapped_keys: Vec<DataLayerM8WrappedCekInput>,
    /// True when legal hold is active and shredding is blocked.
    pub legal_hold_active: bool,
    /// Timestamp when crypto-shredding completed.
    pub shredded_at_epoch_seconds: Option<u64>,
    /// Stable reason marker for shredding transition.
    pub shred_reason_code: Option<&'static str>,
    /// Append-order sequence within owner scope.
    pub sequence: u64,
}

/// Owner scope query for retention worker candidate projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8OwnerScopeQuery {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
}

/// Legal-hold mutation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8LegalHoldRequest {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Message identifier.
    pub message_id: String,
    /// Legal-hold active flag to apply.
    pub legal_hold_active: bool,
}

/// Crypto-shred mutation request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8CryptoShredRequest {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Message identifier.
    pub message_id: String,
    /// Shredding execution timestamp in epoch seconds.
    pub shredded_at_epoch_seconds: u64,
}

/// Retention-due candidate projected for retention worker execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM8RetentionDueCandidate {
    /// Owner DID scope.
    pub owner_did: String,
    /// Message identifier.
    pub message_id: String,
    /// Effective retention class.
    pub retention_class: DataLayerM8RetentionClass,
    /// Calculated due timestamp in epoch seconds.
    pub due_at_epoch_seconds: u64,
    /// Stable reason marker.
    pub reason_code: &'static str,
}

/// M8 compliance registry for owner-scoped retention and shredding controls.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM8ComplianceRegistry {
    messages_by_owner: BTreeMap<String, Vec<DataLayerM8MessageRecord>>,
}

impl DataLayerM8ComplianceRegistry {
    /// Creates an empty M8 compliance registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one owner-scoped message lifecycle record.
    pub fn register_message(
        &mut self,
        input: DataLayerM8MessageRecordInput,
    ) -> Result<DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
        validate_kamn_did(input.owner_did.as_str())?;
        validate_non_empty(input.message_id.as_str(), "message_id")?;
        validate_non_empty(input.content_hash.as_str(), "content_hash")?;
        validate_non_empty(input.hash_chain_prev.as_str(), "hash_chain_prev")?;
        if input.created_at_epoch_seconds == 0 {
            return Err(DataLayerM8ComplianceError::EmptyField(
                "created_at_epoch_seconds",
            ));
        }
        validate_wrapped_keys(&input.wrapped_keys)?;

        let owner_records = self
            .messages_by_owner
            .entry(input.owner_did.clone())
            .or_default();
        if owner_records
            .iter()
            .any(|record| record.message_id == input.message_id)
        {
            return Err(DataLayerM8ComplianceError::DuplicateMessageId {
                owner_did: input.owner_did,
                message_id: input.message_id,
            });
        }

        let mut wrapped_keys = input.wrapped_keys;
        wrapped_keys.sort_by(|left, right| {
            left.recipient_did
                .cmp(&right.recipient_did)
                .then(left.wrapped_cek.cmp(&right.wrapped_cek))
        });

        let record = DataLayerM8MessageRecord {
            owner_did: input.owner_did,
            message_id: input.message_id,
            created_at_epoch_seconds: input.created_at_epoch_seconds,
            content_hash: input.content_hash,
            hash_chain_prev: input.hash_chain_prev,
            retention_class: input.retention_class,
            retention_extension_seconds: input.retention_extension_seconds,
            wrapped_keys,
            legal_hold_active: matches!(
                input.retention_class,
                DataLayerM8RetentionClass::LegalHold
            ),
            shredded_at_epoch_seconds: None,
            shred_reason_code: None,
            sequence: owner_records.len() as u64 + 1,
        };
        owner_records.push(record.clone());
        Ok(record)
    }

    /// Returns retention-due candidates for an owner at `now_epoch_seconds`.
    pub fn retention_due_for_owner(
        &self,
        query: DataLayerM8OwnerScopeQuery,
        now_epoch_seconds: u64,
    ) -> Result<Vec<DataLayerM8RetentionDueCandidate>, DataLayerM8ComplianceError> {
        authorize_owner_scope(query.requester_owner_did.as_str(), query.owner_did.as_str())?;
        if now_epoch_seconds == 0 {
            return Err(DataLayerM8ComplianceError::EmptyField("now_epoch_seconds"));
        }
        let owner_records = self.owner_records_or_error(query.owner_did.as_str())?;

        let mut candidates = owner_records
            .iter()
            .filter_map(|record| {
                if record.shredded_at_epoch_seconds.is_some() || record.legal_hold_active {
                    return None;
                }
                let base_window_seconds = record.retention_class.retention_window_seconds()?;
                let due_at_epoch_seconds = record
                    .created_at_epoch_seconds
                    .saturating_add(base_window_seconds)
                    .saturating_add(record.retention_extension_seconds);
                if now_epoch_seconds >= due_at_epoch_seconds {
                    Some(DataLayerM8RetentionDueCandidate {
                        owner_did: record.owner_did.clone(),
                        message_id: record.message_id.clone(),
                        retention_class: record.retention_class,
                        due_at_epoch_seconds,
                        reason_code: DATA_LAYER_M8_RETENTION_DUE_REASON_CODE,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| {
            left.due_at_epoch_seconds
                .cmp(&right.due_at_epoch_seconds)
                .then(left.message_id.cmp(&right.message_id))
        });
        Ok(candidates)
    }

    /// Applies or releases legal-hold status for one message.
    pub fn set_legal_hold(
        &mut self,
        request: DataLayerM8LegalHoldRequest,
    ) -> Result<DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
        authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )?;
        let message =
            self.owner_message_mut(request.owner_did.as_str(), request.message_id.as_str())?;
        if message.shredded_at_epoch_seconds.is_some() {
            return Err(DataLayerM8ComplianceError::AlreadyShredded {
                message_id: message.message_id.clone(),
            });
        }
        message.legal_hold_active = request.legal_hold_active;
        Ok(message.clone())
    }

    /// Executes crypto-shredding for one message.
    pub fn crypto_shred(
        &mut self,
        request: DataLayerM8CryptoShredRequest,
    ) -> Result<DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
        authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )?;
        if request.shredded_at_epoch_seconds == 0 {
            return Err(DataLayerM8ComplianceError::EmptyField(
                "shredded_at_epoch_seconds",
            ));
        }
        let message =
            self.owner_message_mut(request.owner_did.as_str(), request.message_id.as_str())?;
        if message.legal_hold_active {
            return Err(DataLayerM8ComplianceError::LegalHoldActive {
                message_id: message.message_id.clone(),
            });
        }
        if message.shredded_at_epoch_seconds.is_some() {
            return Err(DataLayerM8ComplianceError::AlreadyShredded {
                message_id: message.message_id.clone(),
            });
        }

        message.wrapped_keys = vec![DataLayerM8WrappedCekInput {
            recipient_did: "m8:crypto-shred:tombstone".to_owned(),
            wrapped_cek: DATA_LAYER_M8_CEK_TOMBSTONE_MARKER.to_owned(),
        }];
        message.shredded_at_epoch_seconds = Some(request.shredded_at_epoch_seconds);
        message.shred_reason_code = Some(DATA_LAYER_M8_CRYPTO_SHRED_REASON_CODE);
        Ok(message.clone())
    }

    /// Returns one message record by owner + message id.
    pub fn message_for_owner(
        &self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<&DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
        let owner_records = self.owner_records_or_error(owner_did)?;
        owner_records
            .iter()
            .find(|record| record.message_id == message_id)
            .ok_or_else(|| DataLayerM8ComplianceError::MessageNotFound {
                owner_did: owner_did.to_owned(),
                message_id: message_id.to_owned(),
            })
    }

    fn owner_records_or_error(
        &self,
        owner_did: &str,
    ) -> Result<&[DataLayerM8MessageRecord], DataLayerM8ComplianceError> {
        validate_kamn_did(owner_did)?;
        self.messages_by_owner
            .get(owner_did)
            .map(Vec::as_slice)
            .ok_or_else(|| DataLayerM8ComplianceError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            })
    }

    fn owner_message_mut(
        &mut self,
        owner_did: &str,
        message_id: &str,
    ) -> Result<&mut DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
        validate_kamn_did(owner_did)?;
        validate_non_empty(message_id, "message_id")?;
        let owner_records = self.messages_by_owner.get_mut(owner_did).ok_or_else(|| {
            DataLayerM8ComplianceError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            }
        })?;
        owner_records
            .iter_mut()
            .find(|record| record.message_id == message_id)
            .ok_or_else(|| DataLayerM8ComplianceError::MessageNotFound {
                owner_did: owner_did.to_owned(),
                message_id: message_id.to_owned(),
            })
    }
}

/// Error taxonomy for M8 compliance lifecycle contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM8ComplianceError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID failed validation.
    InvalidDid(String),
    /// Wrapped key set is empty.
    EmptyWrappedKeys,
    /// Wrapped key input failed validation.
    InvalidWrappedKey(&'static str),
    /// Wrapped key set contains duplicate recipient identities.
    DuplicateWrappedKeyRecipient {
        /// Duplicate recipient DID.
        recipient_did: String,
    },
    /// Owner scope was not found.
    OwnerNotFound {
        /// Missing owner DID.
        owner_did: String,
    },
    /// Message was not found within owner scope.
    MessageNotFound {
        /// Owner DID scope.
        owner_did: String,
        /// Missing message identifier.
        message_id: String,
    },
    /// Duplicate message id registration within owner scope.
    DuplicateMessageId {
        /// Owner DID scope.
        owner_did: String,
        /// Duplicate message identifier.
        message_id: String,
    },
    /// Owner scope authorization failed.
    OwnerScopeViolation {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Legal hold blocks crypto-shredding.
    LegalHoldActive {
        /// Message identifier blocked by legal hold.
        message_id: String,
    },
    /// Message has already been shredded.
    AlreadyShredded {
        /// Message identifier that was already shredded.
        message_id: String,
    },
}

impl fmt::Display for DataLayerM8ComplianceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::EmptyWrappedKeys => write!(f, "wrapped key set must not be empty"),
            Self::InvalidWrappedKey(field) => write!(f, "invalid wrapped key field: {field}"),
            Self::DuplicateWrappedKeyRecipient { recipient_did } => {
                write!(f, "duplicate wrapped key recipient: {recipient_did}")
            }
            Self::OwnerNotFound { owner_did } => write!(f, "owner not found: {owner_did}"),
            Self::MessageNotFound {
                owner_did,
                message_id,
            } => write!(f, "message not found for owner {owner_did}: {message_id}"),
            Self::DuplicateMessageId {
                owner_did,
                message_id,
            } => write!(
                f,
                "duplicate message id for owner {owner_did}: {message_id}"
            ),
            Self::OwnerScopeViolation { reason_code } => {
                write!(f, "owner scope violation: {reason_code}")
            }
            Self::LegalHoldActive { message_id } => {
                write!(f, "legal hold active for message: {message_id}")
            }
            Self::AlreadyShredded { message_id } => {
                write!(f, "message already shredded: {message_id}")
            }
        }
    }
}

impl std::error::Error for DataLayerM8ComplianceError {}

fn validate_non_empty(value: &str, field: &'static str) -> Result<(), DataLayerM8ComplianceError> {
    if value.trim().is_empty() {
        return Err(DataLayerM8ComplianceError::EmptyField(field));
    }
    Ok(())
}

fn validate_kamn_did(value: &str) -> Result<(), DataLayerM8ComplianceError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM8ComplianceError::InvalidDid(value.to_owned()));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM8ComplianceError::InvalidDid(value.to_owned()));
    }
    Ok(())
}

fn validate_wrapped_keys(
    wrapped_keys: &[DataLayerM8WrappedCekInput],
) -> Result<(), DataLayerM8ComplianceError> {
    if wrapped_keys.is_empty() {
        return Err(DataLayerM8ComplianceError::EmptyWrappedKeys);
    }
    let mut seen_recipients = BTreeSet::new();
    for key in wrapped_keys {
        validate_kamn_did(key.recipient_did.as_str())?;
        if !seen_recipients.insert(key.recipient_did.as_str()) {
            return Err(DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient {
                recipient_did: key.recipient_did.clone(),
            });
        }
        if key.wrapped_cek.trim().is_empty() {
            return Err(DataLayerM8ComplianceError::InvalidWrappedKey("wrapped_cek"));
        }
    }
    Ok(())
}

fn authorize_owner_scope(
    requester_owner_did: &str,
    owner_did: &str,
) -> Result<(), DataLayerM8ComplianceError> {
    validate_kamn_did(requester_owner_did)?;
    validate_kamn_did(owner_did)?;
    if requester_owner_did != owner_did {
        return Err(DataLayerM8ComplianceError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE,
        });
    }
    Ok(())
}
