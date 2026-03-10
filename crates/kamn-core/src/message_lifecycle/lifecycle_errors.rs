use super::lifecycle_types::MessageStatus;
use crate::ZkDesignError;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Lifecycle domain validation errors.
pub enum MessageLifecycleError {
    /// The message identifier was empty after trimming.
    EmptyMessageId,
    /// The message identifier already exists in the store.
    DuplicateMessageId(String),
    /// The sender DID failed validation.
    InvalidSenderDid(String),
    /// The recipients collection was empty.
    EmptyRecipients,
    /// One recipient DID failed validation.
    InvalidRecipientDid(String),
    /// A required timestamp field was empty.
    EmptyTimestamp(&'static str),
    /// The expiry timestamp was not strictly after the creation timestamp.
    InvalidExpiryWindow {
        /// The recorded creation timestamp.
        created: String,
        /// The recorded expiry timestamp.
        expires: String,
    },
    /// The requested message identifier was not found.
    NotFound(String),
    /// The requested status transition violates lifecycle policy.
    InvalidTransition {
        /// The current message status.
        from: MessageStatus,
        /// The requested next message status.
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
    /// Lifecycle validation failed before proof admission.
    Lifecycle(MessageLifecycleError),
    /// The message was not in the delivered state required for proof admission.
    InvalidValidationState {
        /// The current message status that blocked proof admission.
        found: MessageStatus,
    },
    /// The proof artifact failed evaluator verification.
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
    /// The snapshot schema version does not match the current parser version.
    SnapshotVersionMismatch {
        /// The schema version expected by the parser.
        expected: u16,
        /// The schema version found in the payload.
        found: u16,
    },
    /// The snapshot contained duplicate message identifiers.
    DuplicateMessageId(String),
    /// The snapshot payload violated lifecycle snapshot invariants.
    InvalidSnapshot(String),
    /// Lifecycle validation failed while restoring a snapshot record.
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
    /// The underlying store returned an I/O error.
    Io(String),
    /// The encoded snapshot payload could not be parsed safely.
    InvalidPayload(String),
    /// A decoded snapshot failed lifecycle restoration validation.
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
