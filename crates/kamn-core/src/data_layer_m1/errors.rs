use crate::kolme_runtime_commit::KolmeRuntimeCommitError;
use std::fmt;

/// Public contract enum for Data Layer M1 Error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1Error {
    /// Empty batch variant for this public contract enum.
    EmptyBatch,
    /// Empty field variant for this public contract enum.
    EmptyField(&'static str),
    /// Invalid content hash variant for this public contract enum.
    InvalidContentHash(String),
    /// Duplicate leaf index variant for this public contract enum.
    DuplicateLeafIndex {
        /// U32 carried by this public contract model.
        leaf_index: u32,
    },
    /// Non contiguous leaf indexes variant for this public contract enum.
    NonContiguousLeafIndexes {
        /// U32 carried by this public contract model.
        expected: u32,
        /// U32 carried by this public contract model.
        found: u32,
    },
    /// Duplicate message id variant for this public contract enum.
    DuplicateMessageId(String),
    /// Unknown message id variant for this public contract enum.
    UnknownMessageId(String),
    /// Invalid merkle proof variant for this public contract enum.
    InvalidMerkleProof(&'static str),
    /// Invalid actor did variant for this public contract enum.
    InvalidActorDid(String),
    /// Invalid anchoring state variant for this public contract enum.
    InvalidAnchoringState(&'static str),
    /// Conflicting anchoring idempotency key variant for this public contract enum.
    ConflictingAnchoringIdempotencyKey {
        /// String carried by this public contract model.
        batch_id: String,
        /// String carried by this public contract model.
        existing_key: String,
        /// String carried by this public contract model.
        provided_key: String,
    },
    /// Invalid failure matrix input variant for this public contract enum.
    InvalidFailureMatrixInput(&'static str),
    /// Kolme runtime commit variant for this public contract enum.
    KolmeRuntimeCommit(KolmeRuntimeCommitError),
}

impl fmt::Display for DataLayerM1Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBatch => write!(f, "merkle batch must contain at least one leaf"),
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidContentHash(value) => write!(f, "invalid content hash: {value}"),
            Self::DuplicateLeafIndex { leaf_index } => {
                write!(f, "duplicate leaf index: {leaf_index}")
            }
            Self::NonContiguousLeafIndexes { expected, found } => {
                write!(f, "non-contiguous leaf indexes: expected {expected}, found {found}")
            }
            Self::DuplicateMessageId(message_id) => {
                write!(f, "duplicate message id in batch: {message_id}")
            }
            Self::UnknownMessageId(message_id) => write!(f, "unknown message id: {message_id}"),
            Self::InvalidMerkleProof(reason) => write!(f, "invalid merkle proof: {reason}"),
            Self::InvalidActorDid(actor_did) => write!(f, "invalid actor did: {actor_did}"),
            Self::InvalidAnchoringState(reason) => write!(f, "invalid anchoring state: {reason}"),
            Self::ConflictingAnchoringIdempotencyKey {
                batch_id,
                existing_key,
                provided_key,
            } => write!(
                f,
                "conflicting anchoring idempotency key for batch {batch_id}; existing {existing_key}, provided {provided_key}"
            ),
            Self::InvalidFailureMatrixInput(field) => {
                write!(f, "invalid anchor failure matrix input: {field}")
            }
            Self::KolmeRuntimeCommit(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for DataLayerM1Error {}

impl From<KolmeRuntimeCommitError> for DataLayerM1Error {
    fn from(value: KolmeRuntimeCommitError) -> Self {
        Self::KolmeRuntimeCommit(value)
    }
}
