use std::fmt;

/// Error taxonomy for M8 compliance lifecycle contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM8ComplianceError {
    /// Empty field variant for this public contract enum.
    EmptyField(&'static str),
    /// Invalid did variant for this public contract enum.
    InvalidDid(String),
    /// Empty wrapped keys variant for this public contract enum.
    EmptyWrappedKeys,
    /// Invalid wrapped key variant for this public contract enum.
    InvalidWrappedKey(&'static str),
    /// Duplicate wrapped key recipient variant for this public contract enum.
    DuplicateWrappedKeyRecipient {
        /// String carried by this public contract model.
        recipient_did: String,
    },
    /// Owner not found variant for this public contract enum.
    OwnerNotFound {
        /// String carried by this public contract model.
        owner_did: String,
    },
    /// Message not found variant for this public contract enum.
    MessageNotFound {
        /// String carried by this public contract model.
        owner_did: String,
        /// String carried by this public contract model.
        message_id: String,
    },
    /// Duplicate message id variant for this public contract enum.
    DuplicateMessageId {
        /// String carried by this public contract model.
        owner_did: String,
        /// String carried by this public contract model.
        message_id: String,
    },
    /// Owner scope violation variant for this public contract enum.
    OwnerScopeViolation {
        /// Str carried by this public contract model.
        reason_code: &'static str,
    },
    /// Legal hold active variant for this public contract enum.
    LegalHoldActive {
        /// String carried by this public contract model.
        message_id: String,
    },
    /// Already shredded variant for this public contract enum.
    AlreadyShredded {
        /// String carried by this public contract model.
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
