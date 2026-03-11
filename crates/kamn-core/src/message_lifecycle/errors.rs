use super::domain::MessageStatus;
use crate::ZkDesignError;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Lifecycle domain validation errors.
pub enum MessageLifecycleError {
    /// Message id is empty.
    EmptyMessageId,
    /// Message id already exists in the store.
    DuplicateMessageId(String),
    /// Sender DID is invalid.
    InvalidSenderDid(String),
    /// Recipient list is empty.
    EmptyRecipients,
    /// One of the recipient DIDs is invalid.
    InvalidRecipientDid(String),
    /// Timestamp field is empty.
    EmptyTimestamp(&'static str),
    /// Expiry is not strictly after creation time.
    InvalidExpiryWindow {
        /// Creation timestamp that failed validation.
        created: String,
        /// Expiry timestamp that failed validation.
        expires: String,
    },
    /// Requested message id does not exist.
    NotFound(String),
    /// Lifecycle transition edge is not permitted.
    InvalidTransition {
        /// Current status.
        from: MessageStatus,
        /// Requested next status.
        to: MessageStatus,
    },
}

impl fmt::Display for MessageLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMessageId => write!(f, "message_id must not be empty"),
            Self::DuplicateMessageId(value) => write!(f, "duplicate message id: {value}"),
            Self::InvalidSenderDid(value) => write!(f, "invalid sender did: {value}"),
            Self::EmptyRecipients => write!(f, "recipients must not be empty"),
            Self::InvalidRecipientDid(value) => write!(f, "invalid recipient did: {value}"),
            Self::EmptyTimestamp(field) => write!(f, "{field} timestamp must not be empty"),
            Self::InvalidExpiryWindow { created, expires } => {
                write!(
                    f,
                    "invalid message expiry window, created {created}, expires {expires}"
                )
            }
            Self::NotFound(value) => write!(f, "message not found: {value}"),
            Self::InvalidTransition { from, to } => {
                write!(
                    f,
                    "invalid message lifecycle transition from {from:?} to {to:?}"
                )
            }
        }
    }
}

impl std::error::Error for MessageLifecycleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Proof admission errors while validating delivered messages.
pub enum MessageProofAdmissionError {
    /// Lifecycle preconditions were not satisfied.
    Lifecycle(MessageLifecycleError),
    /// Message was not in the delivered state.
    InvalidValidationState {
        /// Lifecycle status observed at validation time.
        found: MessageStatus,
    },
    /// Processor-proof verification failed.
    Proof(ZkDesignError),
}

impl fmt::Display for MessageProofAdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lifecycle(error) => write!(f, "{error}"),
            Self::InvalidValidationState { found } => write!(
                f,
                "message must be in Delivered state before processor proof validation (found {found:?})"
            ),
            Self::Proof(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MessageProofAdmissionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Snapshot parsing/restoration errors for lifecycle state.
pub enum MessageLifecycleSnapshotError {
    /// Snapshot schema version does not match the current implementation.
    SnapshotVersionMismatch {
        /// Schema version expected by the current binary.
        expected: u16,
        /// Schema version present in the snapshot payload.
        found: u16,
    },
    /// Snapshot contains the same message id more than once.
    DuplicateMessageId(String),
    /// Snapshot payload violated semantic validation rules.
    InvalidSnapshot(String),
    /// Lifecycle validation failed while rebuilding records.
    Lifecycle(MessageLifecycleError),
}

impl fmt::Display for MessageLifecycleSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SnapshotVersionMismatch { expected, found } => write!(
                f,
                "message lifecycle snapshot version mismatch: expected {expected}, found {found}"
            ),
            Self::DuplicateMessageId(value) => {
                write!(f, "duplicate message id in snapshot: {value}")
            }
            Self::InvalidSnapshot(value) => {
                write!(f, "invalid message lifecycle snapshot: {value}")
            }
            Self::Lifecycle(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MessageLifecycleSnapshotError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Storage-layer errors for lifecycle snapshot persistence.
pub enum MessageLifecycleSnapshotStoreError {
    /// Underlying storage I/O failed.
    Io(String),
    /// Persisted payload is malformed.
    InvalidPayload(String),
    /// Snapshot parse or restore failed.
    Snapshot(MessageLifecycleSnapshotError),
}

impl fmt::Display for MessageLifecycleSnapshotStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(value) => write!(f, "message lifecycle snapshot store I/O error: {value}"),
            Self::InvalidPayload(value) => {
                write!(
                    f,
                    "message lifecycle snapshot store invalid payload: {value}"
                )
            }
            Self::Snapshot(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for MessageLifecycleSnapshotStoreError {}
