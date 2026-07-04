use std::fmt;

/// Error taxonomy for M5 vector-layer contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM5VectorIntegrationError {
    /// Empty field variant for this public contract enum.
    EmptyField(&'static str),
    /// Invalid did variant for this public contract enum.
    InvalidDid(String),
    /// Invalid agent did variant for this public contract enum.
    InvalidAgentDid {
        /// Str carried by this public contract model.
        reason_code: &'static str,
        /// String carried by this public contract model.
        detail: String,
    },
    /// Duplicate embedding id variant for this public contract enum.
    DuplicateEmbeddingId(String),
    /// Invalid vector dimensions variant for this public contract enum.
    InvalidVectorDimensions {
        /// Usize carried by this public contract model.
        expected: usize,
        /// Usize carried by this public contract model.
        found: usize,
    },
    /// Invalid vector value variant for this public contract enum.
    InvalidVectorValue(&'static str),
    /// Invalid limit variant for this public contract enum.
    InvalidLimit(usize),
    /// Invalid lookback window variant for this public contract enum.
    InvalidLookbackWindow(usize),
    /// Owner not found variant for this public contract enum.
    OwnerNotFound {
        /// String carried by this public contract model.
        owner_did: String,
    },
    /// Semantic query unavailable variant for this public contract enum.
    SemanticQueryUnavailable {
        /// Str carried by this public contract model.
        reason_code: &'static str,
    },
    /// Anomaly evaluation unavailable variant for this public contract enum.
    AnomalyEvaluationUnavailable {
        /// Str carried by this public contract model.
        reason_code: &'static str,
    },
    /// Privacy mode violation variant for this public contract enum.
    PrivacyModeViolation {
        /// Str carried by this public contract model.
        reason_code: &'static str,
    },
    /// Insufficient agent history variant for this public contract enum.
    InsufficientAgentHistory {
        /// String carried by this public contract model.
        owner_did: String,
        /// String carried by this public contract model.
        agent_did: String,
    },
    /// Invalid embedding hash chain variant for this public contract enum.
    InvalidEmbeddingHashChain {
        /// String carried by this public contract model.
        owner_did: String,
        /// Usize carried by this public contract model.
        position: usize,
        /// Str carried by this public contract model.
        reason: &'static str,
    },
    /// Embedding sequence not found variant for this public contract enum.
    EmbeddingSequenceNotFound {
        /// String carried by this public contract model.
        owner_did: String,
        /// U64 carried by this public contract model.
        sequence: u64,
    },
}

impl fmt::Display for DataLayerM5VectorIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::InvalidAgentDid {
                reason_code,
                detail,
            } => write!(f, "invalid agent did: {reason_code} ({detail})"),
            Self::DuplicateEmbeddingId(embedding_id) => {
                write!(f, "duplicate embedding_id: {embedding_id}")
            }
            Self::InvalidVectorDimensions { expected, found } => write!(
                f,
                "invalid vector dimensions: expected {expected}, found {found}"
            ),
            Self::InvalidVectorValue(field) => write!(f, "invalid vector value for {field}"),
            Self::InvalidLimit(limit) => write!(f, "invalid limit: {limit}"),
            Self::InvalidLookbackWindow(window) => write!(f, "invalid lookback window: {window}"),
            Self::OwnerNotFound { owner_did } => write!(f, "owner not found: {owner_did}"),
            Self::SemanticQueryUnavailable { reason_code } => {
                write!(f, "semantic query unavailable: {reason_code}")
            }
            Self::AnomalyEvaluationUnavailable { reason_code } => {
                write!(f, "anomaly evaluation unavailable: {reason_code}")
            }
            Self::PrivacyModeViolation { reason_code } => {
                write!(f, "privacy mode violation: {reason_code}")
            }
            Self::InsufficientAgentHistory {
                owner_did,
                agent_did,
            } => write!(
                f,
                "insufficient agent history for owner {owner_did}, agent {agent_did}"
            ),
            Self::InvalidEmbeddingHashChain {
                owner_did,
                position,
                reason,
            } => write!(
                f,
                "invalid embedding hash chain for {owner_did} at {position}: {reason}"
            ),
            Self::EmbeddingSequenceNotFound {
                owner_did,
                sequence,
            } => write!(
                f,
                "embedding sequence not found for owner {owner_did}: {sequence}"
            ),
        }
    }
}

impl std::error::Error for DataLayerM5VectorIntegrationError {}
