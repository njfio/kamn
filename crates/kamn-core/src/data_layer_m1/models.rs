use crate::kolme_runtime_commit::{KolmeCommitReceiptFinality, KolmeRuntimeCommitError};
use std::fmt;

pub const DATA_LAYER_M1_HASH_ALGORITHM: &str = "sha256";
pub const DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE: &str =
    "m1_merkle_proof_verification_valid";
pub const DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE: &str =
    "m1_merkle_proof_verification_invalid";
pub const DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE: &str =
    "m1_anchor_failure_matrix_stable";
pub const DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE: &str =
    "m1_anchor_failure_matrix_drift_detected";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1MerkleLeaf {
    pub message_id: String,
    pub leaf_index: u32,
    pub content_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM1ProofSiblingSide {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1MerkleProofStep {
    pub sibling_hash: String,
    pub sibling_side: DataLayerM1ProofSiblingSide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1MerkleInclusionProof {
    pub batch_id: String,
    pub merkle_root: String,
    pub message_id: String,
    pub leaf_index: u32,
    pub content_hash: String,
    pub leaf_hash: String,
    pub steps: Vec<DataLayerM1MerkleProofStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM1AnchorRetryClass {
    NewSubmission,
    RetryableInFlight,
    FinalizedNoRetry,
    ConflictNoRetry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchorReceipt {
    pub provider: String,
    pub transaction_id: String,
    pub finality: KolmeCommitReceiptFinality,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1AnchorOutcome {
    Submitted(DataLayerM1AnchorReceipt),
    Duplicate(DataLayerM1AnchorReceipt),
    Rejected { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchorResult {
    pub batch_id: String,
    pub idempotency_key: String,
    pub retry_class: DataLayerM1AnchorRetryClass,
    pub outcome: DataLayerM1AnchorOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM1AnchorOutcomeKind {
    Submitted,
    Duplicate,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchorFailureMatrixCase {
    pub case_id: String,
    pub result: DataLayerM1AnchorResult,
    pub expected_retry_class: DataLayerM1AnchorRetryClass,
    pub expected_outcome_kind: DataLayerM1AnchorOutcomeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchorFailureMatrixEvidence {
    pub case_id: String,
    pub batch_id: String,
    pub idempotency_key: String,
    pub expected_retry_class: DataLayerM1AnchorRetryClass,
    pub observed_retry_class: DataLayerM1AnchorRetryClass,
    pub expected_outcome_kind: DataLayerM1AnchorOutcomeKind,
    pub observed_outcome_kind: DataLayerM1AnchorOutcomeKind,
    pub mismatch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1AnchorFailureMatrixDecision {
    Stable { reason_code: &'static str },
    DriftDetected { reason_code: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchorFailureMatrixReport {
    pub decision: DataLayerM1AnchorFailureMatrixDecision,
    pub evidence: Vec<DataLayerM1AnchorFailureMatrixEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1Error {
    EmptyBatch,
    EmptyField(&'static str),
    InvalidContentHash(String),
    DuplicateLeafIndex {
        leaf_index: u32,
    },
    NonContiguousLeafIndexes {
        expected: u32,
        found: u32,
    },
    DuplicateMessageId(String),
    UnknownMessageId(String),
    InvalidMerkleProof(&'static str),
    InvalidActorDid(String),
    InvalidAnchoringState(&'static str),
    ConflictingAnchoringIdempotencyKey {
        batch_id: String,
        existing_key: String,
        provided_key: String,
    },
    InvalidFailureMatrixInput(&'static str),
    KolmeRuntimeCommit(KolmeRuntimeCommitError),
}

impl fmt::Display for DataLayerM1Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBatch => write!(f, "merkle batch must contain at least one leaf"),
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidContentHash(value) => write!(f, "invalid content hash: {value}"),
            Self::DuplicateLeafIndex { leaf_index } => write!(f, "duplicate leaf index: {leaf_index}"),
            Self::NonContiguousLeafIndexes { expected, found } => {
                write!(f, "non-contiguous leaf indexes: expected {expected}, found {found}")
            }
            Self::DuplicateMessageId(message_id) => write!(f, "duplicate message id in batch: {message_id}"),
            Self::UnknownMessageId(message_id) => write!(f, "unknown message id: {message_id}"),
            Self::InvalidMerkleProof(reason) => write!(f, "invalid merkle proof: {reason}"),
            Self::InvalidActorDid(actor_did) => write!(f, "invalid actor did: {actor_did}"),
            Self::InvalidAnchoringState(reason) => write!(f, "invalid anchoring state: {reason}"),
            Self::ConflictingAnchoringIdempotencyKey { batch_id, existing_key, provided_key } => write!(
                f,
                "conflicting anchoring idempotency key for batch {batch_id}; existing {existing_key}, provided {provided_key}"
            ),
            Self::InvalidFailureMatrixInput(field) => write!(f, "invalid anchor failure matrix input: {field}"),
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
