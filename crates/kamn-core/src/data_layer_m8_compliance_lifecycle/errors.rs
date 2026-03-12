use std::fmt;

/// Error taxonomy for M8 compliance lifecycle contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM8ComplianceError {
    EmptyField(&'static str),
    InvalidDid(String),
    EmptyWrappedKeys,
    InvalidWrappedKey(&'static str),
    DuplicateWrappedKeyRecipient {
        recipient_did: String,
    },
    OwnerNotFound {
        owner_did: String,
    },
    MessageNotFound {
        owner_did: String,
        message_id: String,
    },
    DuplicateMessageId {
        owner_did: String,
        message_id: String,
    },
    OwnerScopeViolation {
        reason_code: &'static str,
    },
    LegalHoldActive {
        message_id: String,
    },
    AlreadyShredded {
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
