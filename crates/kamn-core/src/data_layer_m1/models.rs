use crate::kolme_runtime_commit::KolmeCommitReceiptFinality;

/// Stable DATA_LAYER_M1_HASH_ALGORITHM marker used by this public contract.
pub const DATA_LAYER_M1_HASH_ALGORITHM: &str = "sha256";
/// Stable DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE marker used by this public contract.
pub const DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE: &str =
    "m1_merkle_proof_verification_valid";
/// Stable DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE marker used by this public contract.
pub const DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE: &str =
    "m1_merkle_proof_verification_invalid";
/// Stable DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE marker used by this public contract.
pub const DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE: &str =
    "m1_anchor_failure_matrix_stable";
/// Stable DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE marker used by this public contract.
pub const DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE: &str =
    "m1_anchor_failure_matrix_drift_detected";

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M1 Merkle Leaf.
pub struct DataLayerM1MerkleLeaf {
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Leaf index carried by this public contract model.
    pub leaf_index: u32,
    /// Content hash carried by this public contract model.
    pub content_hash: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Public contract enum for Data Layer M1 Proof Sibling Side.
pub enum DataLayerM1ProofSiblingSide {
    /// Left variant for this public contract enum.
    Left,
    /// Right variant for this public contract enum.
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M1 Merkle Proof Step.
pub struct DataLayerM1MerkleProofStep {
    /// Sibling hash carried by this public contract model.
    pub sibling_hash: String,
    /// Sibling side carried by this public contract model.
    pub sibling_side: DataLayerM1ProofSiblingSide,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M1 Merkle Inclusion Proof.
pub struct DataLayerM1MerkleInclusionProof {
    /// Batch id carried by this public contract model.
    pub batch_id: String,
    /// Merkle root carried by this public contract model.
    pub merkle_root: String,
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Leaf index carried by this public contract model.
    pub leaf_index: u32,
    /// Content hash carried by this public contract model.
    pub content_hash: String,
    /// Leaf hash carried by this public contract model.
    pub leaf_hash: String,
    /// Steps carried by this public contract model.
    pub steps: Vec<DataLayerM1MerkleProofStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Public contract enum for Data Layer M1 Anchor Retry Class.
pub enum DataLayerM1AnchorRetryClass {
    /// New submission variant for this public contract enum.
    NewSubmission,
    /// Retryable in flight variant for this public contract enum.
    RetryableInFlight,
    /// Finalized no retry variant for this public contract enum.
    FinalizedNoRetry,
    /// Conflict no retry variant for this public contract enum.
    ConflictNoRetry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M1 Anchor Receipt.
pub struct DataLayerM1AnchorReceipt {
    /// Provider carried by this public contract model.
    pub provider: String,
    /// Transaction id carried by this public contract model.
    pub transaction_id: String,
    /// Finality carried by this public contract model.
    pub finality: KolmeCommitReceiptFinality,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract enum for Data Layer M1 Anchor Outcome.
pub enum DataLayerM1AnchorOutcome {
    /// Submitted variant for this public contract enum.
    Submitted(DataLayerM1AnchorReceipt),
    /// Duplicate variant for this public contract enum.
    Duplicate(DataLayerM1AnchorReceipt),
    /// Rejected variant for this public contract enum.
    Rejected {
        /// Reason carried by this enum variant.
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M1 Anchor Result.
pub struct DataLayerM1AnchorResult {
    /// Batch id carried by this public contract model.
    pub batch_id: String,
    /// Idempotency key carried by this public contract model.
    pub idempotency_key: String,
    /// Retry class carried by this public contract model.
    pub retry_class: DataLayerM1AnchorRetryClass,
    /// Outcome carried by this public contract model.
    pub outcome: DataLayerM1AnchorOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Public contract enum for Data Layer M1 Anchor Outcome Kind.
pub enum DataLayerM1AnchorOutcomeKind {
    /// Submitted variant for this public contract enum.
    Submitted,
    /// Duplicate variant for this public contract enum.
    Duplicate,
    /// Rejected variant for this public contract enum.
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M1 Anchor Failure Matrix Case.
pub struct DataLayerM1AnchorFailureMatrixCase {
    /// Case id carried by this public contract model.
    pub case_id: String,
    /// Result carried by this public contract model.
    pub result: DataLayerM1AnchorResult,
    /// Expected retry class carried by this public contract model.
    pub expected_retry_class: DataLayerM1AnchorRetryClass,
    /// Expected outcome kind carried by this public contract model.
    pub expected_outcome_kind: DataLayerM1AnchorOutcomeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M1 Anchor Failure Matrix Evidence.
pub struct DataLayerM1AnchorFailureMatrixEvidence {
    /// Case id carried by this public contract model.
    pub case_id: String,
    /// Batch id carried by this public contract model.
    pub batch_id: String,
    /// Idempotency key carried by this public contract model.
    pub idempotency_key: String,
    /// Expected retry class carried by this public contract model.
    pub expected_retry_class: DataLayerM1AnchorRetryClass,
    /// Observed retry class carried by this public contract model.
    pub observed_retry_class: DataLayerM1AnchorRetryClass,
    /// Expected outcome kind carried by this public contract model.
    pub expected_outcome_kind: DataLayerM1AnchorOutcomeKind,
    /// Observed outcome kind carried by this public contract model.
    pub observed_outcome_kind: DataLayerM1AnchorOutcomeKind,
    /// Mismatch carried by this public contract model.
    pub mismatch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract enum for Data Layer M1 Anchor Failure Matrix Decision.
pub enum DataLayerM1AnchorFailureMatrixDecision {
    /// Stable variant for this public contract enum.
    Stable {
        /// Reason code carried by this enum variant.
        reason_code: &'static str,
    },
    /// Drift detected variant for this public contract enum.
    DriftDetected {
        /// Reason code carried by this enum variant.
        reason_code: &'static str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Data Layer M1 Anchor Failure Matrix Report.
pub struct DataLayerM1AnchorFailureMatrixReport {
    /// Decision carried by this public contract model.
    pub decision: DataLayerM1AnchorFailureMatrixDecision,
    /// Evidence carried by this public contract model.
    pub evidence: Vec<DataLayerM1AnchorFailureMatrixEvidence>,
}
