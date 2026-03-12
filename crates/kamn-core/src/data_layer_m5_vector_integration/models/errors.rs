use std::fmt;

/// Error taxonomy for M5 vector-layer contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM5VectorIntegrationError {
    EmptyField(&'static str),
    InvalidDid(String),
    InvalidAgentDid { reason_code: &'static str, detail: String },
    DuplicateEmbeddingId(String),
    InvalidVectorDimensions { expected: usize, found: usize },
    InvalidVectorValue(&'static str),
    InvalidLimit(usize),
    InvalidLookbackWindow(usize),
    OwnerNotFound { owner_did: String },
    SemanticQueryUnavailable { reason_code: &'static str },
    AnomalyEvaluationUnavailable { reason_code: &'static str },
    PrivacyModeViolation { reason_code: &'static str },
    InsufficientAgentHistory { owner_did: String, agent_did: String },
    InvalidEmbeddingHashChain {
        owner_did: String,
        position: usize,
        reason: &'static str,
    },
    EmbeddingSequenceNotFound { owner_did: String, sequence: u64 },
}

impl fmt::Display for DataLayerM5VectorIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::InvalidAgentDid { reason_code, detail } => write!(f, "invalid agent did: {reason_code} ({detail})"),
            Self::DuplicateEmbeddingId(embedding_id) => write!(f, "duplicate embedding_id: {embedding_id}"),
            Self::InvalidVectorDimensions { expected, found } => write!(f, "invalid vector dimensions: expected {expected}, found {found}"),
            Self::InvalidVectorValue(field) => write!(f, "invalid vector value for {field}"),
            Self::InvalidLimit(limit) => write!(f, "invalid limit: {limit}"),
            Self::InvalidLookbackWindow(window) => write!(f, "invalid lookback window: {window}"),
            Self::OwnerNotFound { owner_did } => write!(f, "owner not found: {owner_did}"),
            Self::SemanticQueryUnavailable { reason_code } => write!(f, "semantic query unavailable: {reason_code}"),
            Self::AnomalyEvaluationUnavailable { reason_code } => write!(f, "anomaly evaluation unavailable: {reason_code}"),
            Self::PrivacyModeViolation { reason_code } => write!(f, "privacy mode violation: {reason_code}"),
            Self::InsufficientAgentHistory { owner_did, agent_did } => write!(f, "insufficient agent history for owner {owner_did}, agent {agent_did}"),
            Self::InvalidEmbeddingHashChain { owner_did, position, reason } => write!(f, "invalid embedding hash chain for {owner_did} at {position}: {reason}"),
            Self::EmbeddingSequenceNotFound { owner_did, sequence } => write!(f, "embedding sequence not found for owner {owner_did}: {sequence}"),
        }
    }
}

impl std::error::Error for DataLayerM5VectorIntegrationError {}
