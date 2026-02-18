//! M1 trust-anchor contracts for merkle batching, proof generation, and Kolme anchoring.
//!
//! This module builds on M0 content-hash records and provides deterministic
//! merkle root assembly, inclusion-proof verification, and an idempotent
//! anchoring worker that targets the existing Kolme runtime-commit client.

use crate::kolme_runtime_commit::{
    KolmeCommitReceiptFinality, KolmeRuntimeCommitClient, KolmeRuntimeCommitError,
    KolmeRuntimeCommitOutcome, KolmeRuntimeCommitReceipt, KolmeRuntimeCommitRequest,
};
use crate::AgentDid;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Hash label used by M1 merkle contracts.
pub const DATA_LAYER_M1_HASH_ALGORITHM: &str = "sha256";
/// Reason marker for successful merkle proof verification decision wrappers.
pub const DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE: &str =
    "m1_merkle_proof_verification_valid";
/// Reason marker for failed merkle proof verification decision wrappers.
pub const DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE: &str =
    "m1_merkle_proof_verification_invalid";
/// Reason marker for anchoring failure-matrix evaluation with no drift.
pub const DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE: &str =
    "m1_anchor_failure_matrix_stable";
/// Reason marker for anchoring failure-matrix evaluation with detected drift.
pub const DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE: &str =
    "m1_anchor_failure_matrix_drift_detected";

/// One message hash leaf that participates in a deterministic merkle batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1MerkleLeaf {
    /// Stable message identifier.
    pub message_id: String,
    /// Canonical zero-based leaf position.
    pub leaf_index: u32,
    /// Content hash from M0 append-only record.
    pub content_hash: String,
}

/// Sibling side for one proof step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM1ProofSiblingSide {
    /// Sibling hash is on the left side of the current hash.
    Left,
    /// Sibling hash is on the right side of the current hash.
    Right,
}

/// One proof step from leaf to root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1MerkleProofStep {
    /// Sibling hash used to build the parent hash.
    pub sibling_hash: String,
    /// Whether sibling is left or right of current hash.
    pub sibling_side: DataLayerM1ProofSiblingSide,
}

/// Deterministic inclusion proof for one message in a merkle batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1MerkleInclusionProof {
    /// Batch identifier that produced this proof.
    pub batch_id: String,
    /// Batch merkle root.
    pub merkle_root: String,
    /// Message identifier this proof corresponds to.
    pub message_id: String,
    /// Canonical leaf index for the message.
    pub leaf_index: u32,
    /// Leaf content hash from storage.
    pub content_hash: String,
    /// Deterministic leaf hash projected into the merkle tree.
    pub leaf_hash: String,
    /// Ordered proof steps leaf -> root.
    pub steps: Vec<DataLayerM1MerkleProofStep>,
}

/// Deterministic merkle batch projection over content hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1MerkleBatch {
    /// Deterministic batch identifier.
    pub batch_id: String,
    /// Batch merkle root.
    pub merkle_root: String,
    /// Number of messages in the batch.
    pub message_count: usize,
    /// First message identifier in canonical leaf order.
    pub first_message_id: String,
    /// Last message identifier in canonical leaf order.
    pub last_message_id: String,
    /// Merkle tree height (leaf level included).
    pub tree_height: u16,
    leaves: Vec<DataLayerM1MerkleLeaf>,
    levels: Vec<Vec<String>>,
}

impl DataLayerM1MerkleBatch {
    /// Assembles a deterministic merkle batch from message leaves.
    pub fn assemble(mut leaves: Vec<DataLayerM1MerkleLeaf>) -> Result<Self, DataLayerM1Error> {
        if leaves.is_empty() {
            return Err(DataLayerM1Error::EmptyBatch);
        }

        leaves.sort_by(|left, right| {
            left.leaf_index
                .cmp(&right.leaf_index)
                .then(left.message_id.cmp(&right.message_id))
        });

        validate_leaves(&leaves)?;

        let mut levels = Vec::new();
        let mut current = leaves.iter().map(leaf_digest).collect::<Vec<_>>();
        levels.push(current.clone());

        while current.len() > 1 {
            let mut next = Vec::with_capacity(current.len().div_ceil(2));
            let mut index = 0usize;
            while index < current.len() {
                let left = current[index].as_str();
                let right = current.get(index + 1).unwrap_or(&current[index]).as_str();
                next.push(node_digest(levels.len() - 1, left, right));
                index += 2;
            }
            levels.push(next.clone());
            current = next;
        }

        let first_message_id = leaves[0].message_id.clone();
        let last_message_id = leaves[leaves.len() - 1].message_id.clone();
        let merkle_root = current[0].clone();
        let batch_id = batch_digest(
            merkle_root.as_str(),
            leaves.len(),
            first_message_id.as_str(),
            last_message_id.as_str(),
        );

        Ok(Self {
            batch_id,
            merkle_root,
            message_count: leaves.len(),
            first_message_id,
            last_message_id,
            tree_height: levels.len() as u16,
            leaves,
            levels,
        })
    }

    /// Returns canonical leaves in deterministic index order.
    pub fn leaves(&self) -> &[DataLayerM1MerkleLeaf] {
        &self.leaves
    }

    /// Builds an inclusion proof for one message in this batch.
    pub fn inclusion_proof(
        &self,
        message_id: &str,
    ) -> Result<DataLayerM1MerkleInclusionProof, DataLayerM1Error> {
        let position = self
            .leaves
            .iter()
            .position(|leaf| leaf.message_id == message_id)
            .ok_or_else(|| DataLayerM1Error::UnknownMessageId(message_id.to_owned()))?;
        let leaf = &self.leaves[position];

        let mut steps = Vec::new();
        let mut node_index = position;
        for level_index in 0..self.levels.len() - 1 {
            let level = &self.levels[level_index];
            let (sibling_index, sibling_side) = if node_index % 2 == 0 {
                let right = if node_index + 1 < level.len() {
                    node_index + 1
                } else {
                    node_index
                };
                (right, DataLayerM1ProofSiblingSide::Right)
            } else {
                (node_index - 1, DataLayerM1ProofSiblingSide::Left)
            };
            steps.push(DataLayerM1MerkleProofStep {
                sibling_hash: level[sibling_index].clone(),
                sibling_side,
            });
            node_index /= 2;
        }

        Ok(DataLayerM1MerkleInclusionProof {
            batch_id: self.batch_id.clone(),
            merkle_root: self.merkle_root.clone(),
            message_id: leaf.message_id.clone(),
            leaf_index: leaf.leaf_index,
            content_hash: leaf.content_hash.clone(),
            leaf_hash: self.levels[0][position].clone(),
            steps,
        })
    }
}

/// Retry classification for M1 anchoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM1AnchorRetryClass {
    /// First submission for this batch.
    NewSubmission,
    /// Duplicate submission while finality is still pending.
    RetryableInFlight,
    /// Batch already has a finalized/accepted anchor receipt.
    FinalizedNoRetry,
    /// Request was rejected or otherwise conflicted.
    ConflictNoRetry,
}

/// Receipt for one merkle-root anchor submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchorReceipt {
    /// Provider identifier.
    pub provider: String,
    /// Provider transaction/commit identifier.
    pub transaction_id: String,
    /// Reported finality status.
    pub finality: KolmeCommitReceiptFinality,
}

/// Anchoring outcome classification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1AnchorOutcome {
    /// New provider submission accepted.
    Submitted(DataLayerM1AnchorReceipt),
    /// Provider recognized duplicate idempotency key.
    Duplicate(DataLayerM1AnchorReceipt),
    /// Provider rejected submission.
    Rejected {
        /// Deterministic rejection reason.
        reason: String,
    },
}

/// Result envelope for anchoring one merkle batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchorResult {
    /// Anchored batch identifier.
    pub batch_id: String,
    /// Deterministic idempotency key.
    pub idempotency_key: String,
    /// Retry classification for this submission attempt.
    pub retry_class: DataLayerM1AnchorRetryClass,
    /// Provider outcome.
    pub outcome: DataLayerM1AnchorOutcome,
}

/// Outcome-kind projection used by anchoring failure-matrix contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM1AnchorOutcomeKind {
    /// Submitted provider outcome.
    Submitted,
    /// Duplicate provider outcome.
    Duplicate,
    /// Rejected provider outcome.
    Rejected,
}

/// One expected anchoring result in deterministic failure-matrix evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchorFailureMatrixCase {
    /// Stable case identifier.
    pub case_id: String,
    /// Observed anchoring result to classify.
    pub result: DataLayerM1AnchorResult,
    /// Expected retry class for this case.
    pub expected_retry_class: DataLayerM1AnchorRetryClass,
    /// Expected outcome kind for this case.
    pub expected_outcome_kind: DataLayerM1AnchorOutcomeKind,
}

/// Per-case failure-matrix evidence entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchorFailureMatrixEvidence {
    /// Stable case identifier.
    pub case_id: String,
    /// Observed batch id.
    pub batch_id: String,
    /// Observed idempotency key.
    pub idempotency_key: String,
    /// Expected retry class.
    pub expected_retry_class: DataLayerM1AnchorRetryClass,
    /// Observed retry class.
    pub observed_retry_class: DataLayerM1AnchorRetryClass,
    /// Expected outcome kind.
    pub expected_outcome_kind: DataLayerM1AnchorOutcomeKind,
    /// Observed outcome kind.
    pub observed_outcome_kind: DataLayerM1AnchorOutcomeKind,
    /// Whether observed values drifted from expectations.
    pub mismatch: bool,
}

/// Aggregate decision marker for anchoring failure-matrix evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1AnchorFailureMatrixDecision {
    /// No mismatch across evaluated cases.
    Stable {
        /// Stable reason code.
        reason_code: &'static str,
    },
    /// At least one case mismatch detected.
    DriftDetected {
        /// Stable reason code.
        reason_code: &'static str,
    },
}

/// Aggregate anchoring failure-matrix report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchorFailureMatrixReport {
    /// Aggregate decision.
    pub decision: DataLayerM1AnchorFailureMatrixDecision,
    /// Per-case mismatch evidence in input order.
    pub evidence: Vec<DataLayerM1AnchorFailureMatrixEvidence>,
}

/// Kolme anchoring worker for deterministic M1 merkle-root submissions.
#[derive(Debug, Clone)]
pub struct DataLayerM1KolmeAnchoringWorker<C> {
    client: C,
    actor_did: AgentDid,
    state_root_prefix: String,
    next_nonce: u64,
    nonce_by_batch_id: BTreeMap<String, u64>,
    idempotency_by_batch_id: BTreeMap<String, String>,
    receipt_by_batch_id: BTreeMap<String, DataLayerM1AnchorReceipt>,
}

impl<C> DataLayerM1KolmeAnchoringWorker<C> {
    /// Creates a new anchoring worker.
    pub fn new(
        client: C,
        actor_did: &str,
        state_root_prefix: &str,
    ) -> Result<Self, DataLayerM1Error> {
        if state_root_prefix.trim().is_empty() {
            return Err(DataLayerM1Error::EmptyField("state_root_prefix"));
        }
        let actor_did = AgentDid::parse(actor_did)
            .map_err(|_| DataLayerM1Error::InvalidActorDid(actor_did.to_owned()))?;

        Ok(Self {
            client,
            actor_did,
            state_root_prefix: state_root_prefix.to_owned(),
            next_nonce: 1,
            nonce_by_batch_id: BTreeMap::new(),
            idempotency_by_batch_id: BTreeMap::new(),
            receipt_by_batch_id: BTreeMap::new(),
        })
    }
}

impl<C> DataLayerM1KolmeAnchoringWorker<C>
where
    C: KolmeRuntimeCommitClient,
{
    /// Anchors one merkle batch via Kolme runtime-commit submission.
    pub fn anchor_batch(
        &mut self,
        batch: &DataLayerM1MerkleBatch,
    ) -> Result<DataLayerM1AnchorResult, DataLayerM1Error> {
        if let Some(receipt) = self.receipt_by_batch_id.get(&batch.batch_id) {
            let idempotency_key = self
                .idempotency_by_batch_id
                .get(&batch.batch_id)
                .cloned()
                .ok_or(DataLayerM1Error::InvalidAnchoringState(
                    "missing idempotency key for accepted receipt",
                ))?;
            return Ok(DataLayerM1AnchorResult {
                batch_id: batch.batch_id.clone(),
                idempotency_key,
                retry_class: DataLayerM1AnchorRetryClass::FinalizedNoRetry,
                outcome: DataLayerM1AnchorOutcome::Duplicate(receipt.clone()),
            });
        }

        let nonce = self.assign_or_resolve_nonce(batch.batch_id.as_str());
        let request = self.build_request(batch, nonce)?;
        let idempotency_key = request.idempotency_key().to_owned();
        self.upsert_idempotency(batch.batch_id.as_str(), idempotency_key.as_str())?;

        let outcome = self.client.submit_commit(&request)?;
        match outcome {
            KolmeRuntimeCommitOutcome::Submitted(receipt) => {
                let mapped = map_receipt(receipt);
                self.receipt_by_batch_id
                    .insert(batch.batch_id.clone(), mapped.clone());
                Ok(DataLayerM1AnchorResult {
                    batch_id: batch.batch_id.clone(),
                    idempotency_key,
                    retry_class: DataLayerM1AnchorRetryClass::NewSubmission,
                    outcome: DataLayerM1AnchorOutcome::Submitted(mapped),
                })
            }
            KolmeRuntimeCommitOutcome::Duplicate(receipt) => {
                let mapped = map_receipt(receipt);
                self.receipt_by_batch_id
                    .insert(batch.batch_id.clone(), mapped.clone());
                let retry_class = if mapped.finality == KolmeCommitReceiptFinality::Pending {
                    DataLayerM1AnchorRetryClass::RetryableInFlight
                } else {
                    DataLayerM1AnchorRetryClass::FinalizedNoRetry
                };
                Ok(DataLayerM1AnchorResult {
                    batch_id: batch.batch_id.clone(),
                    idempotency_key,
                    retry_class,
                    outcome: DataLayerM1AnchorOutcome::Duplicate(mapped),
                })
            }
            KolmeRuntimeCommitOutcome::Rejected { reason } => Ok(DataLayerM1AnchorResult {
                batch_id: batch.batch_id.clone(),
                idempotency_key,
                retry_class: DataLayerM1AnchorRetryClass::ConflictNoRetry,
                outcome: DataLayerM1AnchorOutcome::Rejected { reason },
            }),
        }
    }

    fn assign_or_resolve_nonce(&mut self, batch_id: &str) -> u64 {
        if let Some(existing) = self.nonce_by_batch_id.get(batch_id) {
            return *existing;
        }
        let nonce = self.next_nonce;
        self.next_nonce += 1;
        self.nonce_by_batch_id.insert(batch_id.to_owned(), nonce);
        nonce
    }

    fn upsert_idempotency(
        &mut self,
        batch_id: &str,
        idempotency_key: &str,
    ) -> Result<(), DataLayerM1Error> {
        if let Some(existing) = self.idempotency_by_batch_id.get(batch_id) {
            if existing != idempotency_key {
                return Err(DataLayerM1Error::ConflictingAnchoringIdempotencyKey {
                    batch_id: batch_id.to_owned(),
                    existing_key: existing.clone(),
                    provided_key: idempotency_key.to_owned(),
                });
            }
            return Ok(());
        }
        self.idempotency_by_batch_id
            .insert(batch_id.to_owned(), idempotency_key.to_owned());
        Ok(())
    }

    fn build_request(
        &self,
        batch: &DataLayerM1MerkleBatch,
        nonce: u64,
    ) -> Result<KolmeRuntimeCommitRequest, DataLayerM1Error> {
        let operation_id = format!("data-layer-m1-anchor-{}", batch.batch_id);
        let state_root = format!("{}:{}", self.state_root_prefix, batch.merkle_root);
        let payload_hash = tagged_digest(
            format!(
                "anchor-payload|batch:{}|root:{}|count:{}|first:{}|last:{}",
                batch.batch_id,
                batch.merkle_root,
                batch.message_count,
                batch.first_message_id,
                batch.last_message_id
            )
            .as_str(),
        );
        let request = KolmeRuntimeCommitRequest::deterministic(
            operation_id.as_str(),
            state_root.as_str(),
            self.actor_did.as_str(),
            nonce,
            payload_hash.as_str(),
        )?;
        Ok(request)
    }
}

/// Verifies an inclusion proof fail-closed.
pub fn verify_data_layer_m1_inclusion_proof(
    proof: &DataLayerM1MerkleInclusionProof,
) -> Result<(), DataLayerM1Error> {
    if proof.batch_id.trim().is_empty() {
        return Err(DataLayerM1Error::EmptyField("batch_id"));
    }
    if proof.merkle_root.trim().is_empty() {
        return Err(DataLayerM1Error::EmptyField("merkle_root"));
    }
    if proof.message_id.trim().is_empty() {
        return Err(DataLayerM1Error::EmptyField("message_id"));
    }
    if !is_valid_content_hash(proof.content_hash.as_str()) {
        return Err(DataLayerM1Error::InvalidContentHash(
            proof.content_hash.clone(),
        ));
    }

    let expected_leaf = leaf_digest(&DataLayerM1MerkleLeaf {
        message_id: proof.message_id.clone(),
        leaf_index: proof.leaf_index,
        content_hash: proof.content_hash.clone(),
    });
    if expected_leaf != proof.leaf_hash {
        return Err(DataLayerM1Error::InvalidMerkleProof("leaf hash mismatch"));
    }

    let mut current = proof.leaf_hash.clone();
    for (level_index, step) in proof.steps.iter().enumerate() {
        if step.sibling_hash.trim().is_empty() {
            return Err(DataLayerM1Error::InvalidMerkleProof(
                "proof sibling hash must not be empty",
            ));
        }
        current = match step.sibling_side {
            DataLayerM1ProofSiblingSide::Left => {
                node_digest(level_index, step.sibling_hash.as_str(), current.as_str())
            }
            DataLayerM1ProofSiblingSide::Right => {
                node_digest(level_index, current.as_str(), step.sibling_hash.as_str())
            }
        };
    }

    if current != proof.merkle_root {
        return Err(DataLayerM1Error::InvalidMerkleProof("proof root mismatch"));
    }
    Ok(())
}

/// Deterministic proof-verification decision wrapper with stable reason markers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1ProofVerificationDecision {
    /// Proof verified successfully.
    Valid {
        /// Stable reason code.
        reason_code: &'static str,
    },
    /// Proof verification failed.
    Invalid {
        /// Stable reason code.
        reason_code: &'static str,
        /// Original fail-closed verification error.
        error: DataLayerM1Error,
    },
}

/// Evaluates proof verification and projects a deterministic reason-coded decision.
pub fn evaluate_data_layer_m1_inclusion_proof(
    proof: &DataLayerM1MerkleInclusionProof,
) -> DataLayerM1ProofVerificationDecision {
    match verify_data_layer_m1_inclusion_proof(proof) {
        Ok(()) => DataLayerM1ProofVerificationDecision::Valid {
            reason_code: DATA_LAYER_M1_PROOF_VERIFICATION_VALID_REASON_CODE,
        },
        Err(error) => DataLayerM1ProofVerificationDecision::Invalid {
            reason_code: DATA_LAYER_M1_PROOF_VERIFICATION_INVALID_REASON_CODE,
            error,
        },
    }
}

/// Evaluates deterministic anchoring failure-matrix expectations.
pub fn evaluate_data_layer_m1_anchor_failure_matrix(
    cases: &[DataLayerM1AnchorFailureMatrixCase],
) -> Result<DataLayerM1AnchorFailureMatrixReport, DataLayerM1Error> {
    if cases.is_empty() {
        return Err(DataLayerM1Error::InvalidFailureMatrixInput("cases"));
    }

    let mut evidence = Vec::with_capacity(cases.len());
    for case in cases {
        if case.case_id.trim().is_empty() {
            return Err(DataLayerM1Error::InvalidFailureMatrixInput("case_id"));
        }
        let observed_outcome_kind = anchor_outcome_kind(&case.result.outcome);
        let observed_retry_class = case.result.retry_class;
        let mismatch = observed_retry_class != case.expected_retry_class
            || observed_outcome_kind != case.expected_outcome_kind;
        evidence.push(DataLayerM1AnchorFailureMatrixEvidence {
            case_id: case.case_id.clone(),
            batch_id: case.result.batch_id.clone(),
            idempotency_key: case.result.idempotency_key.clone(),
            expected_retry_class: case.expected_retry_class,
            observed_retry_class,
            expected_outcome_kind: case.expected_outcome_kind,
            observed_outcome_kind,
            mismatch,
        });
    }

    let decision = if evidence.iter().all(|entry| !entry.mismatch) {
        DataLayerM1AnchorFailureMatrixDecision::Stable {
            reason_code: DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_STABLE_REASON_CODE,
        }
    } else {
        DataLayerM1AnchorFailureMatrixDecision::DriftDetected {
            reason_code: DATA_LAYER_M1_ANCHOR_FAILURE_MATRIX_DRIFT_REASON_CODE,
        }
    };

    Ok(DataLayerM1AnchorFailureMatrixReport { decision, evidence })
}

/// Error taxonomy for M1 merkle and anchoring contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1Error {
    /// Merkle batch must include at least one leaf.
    EmptyBatch,
    /// Required field was empty.
    EmptyField(&'static str),
    /// Content hash format is invalid.
    InvalidContentHash(String),
    /// Leaf index is duplicated in the same batch.
    DuplicateLeafIndex {
        /// Duplicate index value.
        leaf_index: u32,
    },
    /// Leaf index sequence is not contiguous from zero.
    NonContiguousLeafIndexes {
        /// Expected next index.
        expected: u32,
        /// Found index value.
        found: u32,
    },
    /// Message id appears more than once in one batch.
    DuplicateMessageId(String),
    /// Message id is not present in batch.
    UnknownMessageId(String),
    /// Inclusion proof failed verification.
    InvalidMerkleProof(&'static str),
    /// Anchor worker actor DID is invalid.
    InvalidActorDid(String),
    /// Worker internal anchor state is inconsistent.
    InvalidAnchoringState(&'static str),
    /// Idempotency key mismatch for same batch id.
    ConflictingAnchoringIdempotencyKey {
        /// Batch identifier.
        batch_id: String,
        /// Existing idempotency key.
        existing_key: String,
        /// Newly provided idempotency key.
        provided_key: String,
    },
    /// Invalid anchoring failure-matrix input.
    InvalidFailureMatrixInput(&'static str),
    /// Wrapped Kolme runtime commit error.
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
            Self::NonContiguousLeafIndexes { expected, found } => write!(
                f,
                "non-contiguous leaf indexes: expected {expected}, found {found}"
            ),
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

fn validate_leaves(leaves: &[DataLayerM1MerkleLeaf]) -> Result<(), DataLayerM1Error> {
    let mut seen_indexes = BTreeSet::new();
    let mut seen_message_ids = BTreeSet::new();

    for (position, leaf) in leaves.iter().enumerate() {
        if leaf.message_id.trim().is_empty() {
            return Err(DataLayerM1Error::EmptyField("message_id"));
        }
        if !is_valid_content_hash(leaf.content_hash.as_str()) {
            return Err(DataLayerM1Error::InvalidContentHash(
                leaf.content_hash.clone(),
            ));
        }
        if !seen_indexes.insert(leaf.leaf_index) {
            return Err(DataLayerM1Error::DuplicateLeafIndex {
                leaf_index: leaf.leaf_index,
            });
        }
        if !seen_message_ids.insert(leaf.message_id.clone()) {
            return Err(DataLayerM1Error::DuplicateMessageId(
                leaf.message_id.clone(),
            ));
        }

        let expected_index = position as u32;
        if leaf.leaf_index != expected_index {
            return Err(DataLayerM1Error::NonContiguousLeafIndexes {
                expected: expected_index,
                found: leaf.leaf_index,
            });
        }
    }

    Ok(())
}

fn is_valid_content_hash(content_hash: &str) -> bool {
    let trimmed = content_hash.trim();
    trimmed.starts_with("sha256:") && trimmed.len() > "sha256:".len()
}

fn leaf_digest(leaf: &DataLayerM1MerkleLeaf) -> String {
    tagged_digest(
        format!(
            "leaf|index:{}|id:{}|content:{}",
            leaf.leaf_index, leaf.message_id, leaf.content_hash
        )
        .as_str(),
    )
}

fn node_digest(level: usize, left: &str, right: &str) -> String {
    tagged_digest(format!("node|level:{level}|left:{left}|right:{right}").as_str())
}

fn batch_digest(merkle_root: &str, message_count: usize, first: &str, last: &str) -> String {
    tagged_digest(
        format!("batch|root:{merkle_root}|count:{message_count}|first:{first}|last:{last}")
            .as_str(),
    )
}

fn tagged_digest(value: &str) -> String {
    format!(
        "{DATA_LAYER_M1_HASH_ALGORITHM}:{}",
        deterministic_digest_256_hex(value)
    )
}

fn deterministic_digest_256_hex(value: &str) -> String {
    const SEEDS: [u64; 4] = [
        0x243f6a8885a308d3,
        0x13198a2e03707344,
        0xa4093822299f31d0,
        0x082efa98ec4e6c89,
    ];
    let mut output = String::with_capacity(64);
    for (index, seed) in SEEDS.iter().enumerate() {
        let mut acc = *seed ^ (index as u64).wrapping_mul(0x9e3779b97f4a7c15);
        for byte in value.bytes() {
            acc ^= u64::from(byte);
            acc = acc.wrapping_mul(0x00000100000001B3);
            acc ^= acc.rotate_left(13);
        }
        output.push_str(&format!("{acc:016x}"));
    }
    output
}

fn map_receipt(receipt: KolmeRuntimeCommitReceipt) -> DataLayerM1AnchorReceipt {
    DataLayerM1AnchorReceipt {
        provider: receipt.provider,
        transaction_id: receipt.commit_id,
        finality: receipt.finality,
    }
}

fn anchor_outcome_kind(outcome: &DataLayerM1AnchorOutcome) -> DataLayerM1AnchorOutcomeKind {
    match outcome {
        DataLayerM1AnchorOutcome::Submitted(_) => DataLayerM1AnchorOutcomeKind::Submitted,
        DataLayerM1AnchorOutcome::Duplicate(_) => DataLayerM1AnchorOutcomeKind::Duplicate,
        DataLayerM1AnchorOutcome::Rejected { .. } => DataLayerM1AnchorOutcomeKind::Rejected,
    }
}
