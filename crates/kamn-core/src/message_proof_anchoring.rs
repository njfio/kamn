//! Message proof anchoring contracts aligned to message lifecycle transitions.

use crate::kolme_runtime_commit::{
    KolmeRuntimeCommitClient, KolmeRuntimeCommitError, KolmeRuntimeCommitOutcome,
    KolmeRuntimeCommitRequest,
};
use crate::{AgentDid, MessageLifecycleError, MessageLifecycleStore, MessageStatus};
use std::collections::BTreeMap;
use std::fmt;

const MESSAGE_PROOF_ANCHOR_INVALID_ACTOR_DID_REASON_CODE: &str =
    "message_proof_anchor_invalid_actor_did";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Retry classification for message proof anchor submissions.
pub enum MessageProofAnchorRetryClass {
    /// First submission for message/key pair.
    NewSubmission,
    /// Submission in-flight and retry is allowed.
    RetryableInFlight,
    /// Submission already finalized and should not retry.
    FinalizedNoRetry,
    /// Idempotency key conflicts with existing submission.
    ConflictNoRetry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Finality status for message proof anchor submission.
pub enum MessageProofAnchorFinalityStatus {
    /// Submission finalized successfully.
    Confirmed,
    /// Submission finalized as rejected.
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Finality record tracked per message anchor submission.
pub struct MessageProofAnchorFinalityRecord {
    /// Idempotency key associated with submission.
    pub idempotency_key: String,
    /// Monotonic finality sequence number.
    pub sequence: u64,
    /// Finality status.
    pub status: MessageProofAnchorFinalityStatus,
    /// Provider receipt payload for finality event.
    pub receipt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Message proof anchor request envelope.
pub struct MessageProofAnchorRequest {
    /// Stable message identifier.
    pub message_id: String,
    /// Actor DID submitting the anchor operation.
    pub actor_did: String,
    /// Strictly positive anchor submission nonce.
    pub nonce: u64,
    /// Deterministic proof hash marker for off-chain message proof.
    pub proof_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Chain adapter request for message proof anchor submission.
pub struct MessageProofAnchorSubmissionRequest {
    /// Stable message identifier.
    pub message_id: String,
    /// Actor DID submitting the anchor operation.
    pub actor_did: String,
    /// Strictly positive anchor submission nonce.
    pub nonce: u64,
    /// Deterministic proof hash marker for off-chain message proof.
    pub proof_hash: String,
    /// Deterministic idempotency key.
    pub idempotency_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Chain adapter receipt for one proof-anchor submission attempt.
pub struct MessageProofAnchorReceipt {
    /// Provider name that handled submission.
    pub provider: String,
    /// Provider transaction identifier.
    pub transaction_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Outcome returned by message proof chain adapters.
pub enum MessageProofAnchorSubmissionOutcome {
    /// New submission accepted by provider.
    Submitted(MessageProofAnchorReceipt),
    /// Duplicate idempotency key acknowledged with existing receipt.
    Duplicate(MessageProofAnchorReceipt),
    /// Submission rejected by provider policy.
    Rejected {
        /// Provider-supplied deterministic rejection reason.
        reason: String,
    },
    /// Anchor service determined no provider call was needed.
    FinalizedNoOp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Result envelope for proof anchor submission flow.
pub struct MessageProofAnchorResult {
    /// Message id processed by submission flow.
    pub message_id: String,
    /// Idempotency key used for this flow.
    pub idempotency_key: String,
    /// Retry classification returned by anchor service.
    pub retry_class: MessageProofAnchorRetryClass,
    /// Provider/service submission outcome.
    pub outcome: MessageProofAnchorSubmissionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Message proof anchoring error taxonomy.
pub enum MessageProofAnchoringError {
    /// Underlying lifecycle error.
    Lifecycle(MessageLifecycleError),
    /// Message id is empty.
    EmptyMessageId,
    /// Actor DID is empty.
    EmptyActorDid,
    /// Actor DID is invalid.
    InvalidActorDid {
        /// Input field carrying invalid DID.
        field: &'static str,
        /// Stable deterministic reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Anchor nonce is invalid (zero).
    InvalidAnchorNonce(u64),
    /// Proof hash is empty.
    EmptyProofHash,
    /// Message status does not support anchor submission.
    InvalidAnchorState {
        /// Observed status.
        found: MessageStatus,
    },
    /// Provided idempotency key conflicts with existing key for message id.
    ConflictingAnchorIdempotencyKey {
        /// Message identifier.
        message_id: String,
        /// Existing idempotency key recorded by service.
        existing_key: String,
        /// New idempotency key supplied by caller.
        provided_key: String,
    },
    /// Finality update references unknown idempotency key.
    UnknownAnchorIdempotencyKey {
        /// Message identifier.
        message_id: String,
        /// Unrecognized idempotency key.
        idempotency_key: String,
    },
    /// Finality update sequence is older than current sequence.
    StaleFinalityUpdate {
        /// Message identifier.
        message_id: String,
        /// Current accepted sequence.
        current_sequence: u64,
        /// Attempted stale sequence.
        attempted_sequence: u64,
    },
    /// Finality update conflicts with existing record at same/newer sequence.
    ConflictingFinalityUpdate {
        /// Message identifier.
        message_id: String,
        /// Sequence where conflict occurred.
        sequence: u64,
    },
    /// Chain adapter submission failed while preparing or submitting payload.
    ChainAdapterSubmitFailed {
        /// Failure context field.
        context: &'static str,
        /// Failure reason.
        reason: String,
    },
}

impl MessageProofAnchoringError {
    /// Returns stable reason code for telemetry/policy contract lanes.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::Lifecycle(_) => "message_proof_anchor_lifecycle_error",
            Self::EmptyMessageId => "message_proof_anchor_empty_message_id",
            Self::EmptyActorDid => "message_proof_anchor_empty_actor_did",
            Self::InvalidActorDid { .. } => MESSAGE_PROOF_ANCHOR_INVALID_ACTOR_DID_REASON_CODE,
            Self::InvalidAnchorNonce(_) => "message_proof_anchor_invalid_nonce",
            Self::EmptyProofHash => "message_proof_anchor_empty_proof_hash",
            Self::InvalidAnchorState { .. } => "message_proof_anchor_invalid_state",
            Self::ConflictingAnchorIdempotencyKey { .. } => "message_proof_anchor_conflicting_key",
            Self::UnknownAnchorIdempotencyKey { .. } => "message_proof_anchor_unknown_key",
            Self::StaleFinalityUpdate { .. } => "message_proof_anchor_finality_stale",
            Self::ConflictingFinalityUpdate { .. } => "message_proof_anchor_finality_conflict",
            Self::ChainAdapterSubmitFailed { .. } => "message_proof_anchor_chain_submit_failed",
        }
    }
}

impl fmt::Display for MessageProofAnchoringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lifecycle(error) => write!(f, "{error}"),
            Self::EmptyMessageId => write!(f, "message_id must not be empty"),
            Self::EmptyActorDid => write!(f, "actor_did must not be empty"),
            Self::InvalidActorDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::InvalidAnchorNonce(value) => {
                write!(f, "anchor nonce must be positive (found {value})")
            }
            Self::EmptyProofHash => write!(f, "proof_hash must not be empty"),
            Self::InvalidAnchorState { found } => write!(
                f,
                "message must be Broadcast or Included before proof anchoring (found {found:?})"
            ),
            Self::ConflictingAnchorIdempotencyKey {
                message_id,
                existing_key,
                provided_key,
            } => write!(
                f,
                "conflicting anchor idempotency key for message {message_id}; existing {existing_key}, provided {provided_key}"
            ),
            Self::UnknownAnchorIdempotencyKey {
                message_id,
                idempotency_key,
            } => write!(
                f,
                "unknown anchor idempotency key for message {message_id}: {idempotency_key}"
            ),
            Self::StaleFinalityUpdate {
                message_id,
                current_sequence,
                attempted_sequence,
            } => write!(
                f,
                "stale anchor finality update for message {message_id}; current sequence {current_sequence}, attempted {attempted_sequence}"
            ),
            Self::ConflictingFinalityUpdate {
                message_id,
                sequence,
            } => write!(
                f,
                "conflicting anchor finality update for message {message_id} at sequence {sequence}"
            ),
            Self::ChainAdapterSubmitFailed { context, reason } => write!(
                f,
                "message proof anchor chain adapter submission failed for {context}: {reason}"
            ),
        }
    }
}

impl std::error::Error for MessageProofAnchoringError {}

/// Chain adapter abstraction for message proof anchor backends.
pub trait MessageProofChainAdapter {
    /// Submits one message proof anchor request via backing provider.
    fn submit_anchor(
        &mut self,
        request: &MessageProofAnchorSubmissionRequest,
    ) -> Result<MessageProofAnchorSubmissionOutcome, MessageProofAnchoringError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// In-memory proof anchor adapter for deterministic tests and local workflows.
pub struct InMemoryMessageProofChainAdapter {
    provider: String,
    receipts_by_key: BTreeMap<String, MessageProofAnchorReceipt>,
    rejected_reasons_by_key: BTreeMap<String, String>,
}

impl InMemoryMessageProofChainAdapter {
    /// Creates an in-memory proof anchor adapter with provider label.
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_owned(),
            receipts_by_key: BTreeMap::new(),
            rejected_reasons_by_key: BTreeMap::new(),
        }
    }

    /// Configures deterministic rejection for one idempotency key.
    pub fn reject_idempotency_key(&mut self, idempotency_key: &str, reason: &str) {
        self.rejected_reasons_by_key
            .insert(idempotency_key.to_owned(), reason.to_owned());
    }
}

impl MessageProofChainAdapter for InMemoryMessageProofChainAdapter {
    fn submit_anchor(
        &mut self,
        request: &MessageProofAnchorSubmissionRequest,
    ) -> Result<MessageProofAnchorSubmissionOutcome, MessageProofAnchoringError> {
        if let Some(reason) = self.rejected_reasons_by_key.get(&request.idempotency_key) {
            return Ok(MessageProofAnchorSubmissionOutcome::Rejected {
                reason: reason.clone(),
            });
        }

        if let Some(existing) = self.receipts_by_key.get(&request.idempotency_key) {
            return Ok(MessageProofAnchorSubmissionOutcome::Duplicate(
                existing.clone(),
            ));
        }

        let receipt = MessageProofAnchorReceipt {
            provider: self.provider.clone(),
            transaction_id: format!("message-anchor:{}:{}", request.message_id, request.nonce),
        };
        self.receipts_by_key
            .insert(request.idempotency_key.clone(), receipt.clone());
        Ok(MessageProofAnchorSubmissionOutcome::Submitted(receipt))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Kolme-backed proof anchor adapter.
pub struct KolmeMessageProofChainAdapter<C> {
    client: C,
    state_root_prefix: String,
}

impl<C> KolmeMessageProofChainAdapter<C> {
    /// Creates a Kolme-backed proof anchor adapter.
    pub fn new(client: C, state_root_prefix: &str) -> Result<Self, MessageProofAnchoringError> {
        if state_root_prefix.trim().is_empty() {
            return Err(MessageProofAnchoringError::ChainAdapterSubmitFailed {
                context: "state_root_prefix",
                reason: "must not be empty".to_owned(),
            });
        }
        Ok(Self {
            client,
            state_root_prefix: state_root_prefix.to_owned(),
        })
    }

    fn runtime_request_for_anchor(
        &self,
        request: &MessageProofAnchorSubmissionRequest,
    ) -> Result<KolmeRuntimeCommitRequest, MessageProofAnchoringError> {
        let operation_id = format!("message-anchor:{}:{}", request.message_id, request.nonce);
        let state_root = format!("{}:{}", self.state_root_prefix, request.nonce);
        KolmeRuntimeCommitRequest::deterministic(
            operation_id.as_str(),
            state_root.as_str(),
            request.actor_did.as_str(),
            request.nonce,
            request.proof_hash.as_str(),
        )
        .map_err(Self::map_runtime_commit_error)
    }

    fn map_runtime_commit_error(error: KolmeRuntimeCommitError) -> MessageProofAnchoringError {
        match error {
            KolmeRuntimeCommitError::InvalidRequest { field, reason } => {
                MessageProofAnchoringError::ChainAdapterSubmitFailed {
                    context: field,
                    reason: reason.to_owned(),
                }
            }
            _ => MessageProofAnchoringError::ChainAdapterSubmitFailed {
                context: "kolme_runtime_commit",
                reason: error.to_string(),
            },
        }
    }
}

impl<C: KolmeRuntimeCommitClient> MessageProofChainAdapter for KolmeMessageProofChainAdapter<C> {
    fn submit_anchor(
        &mut self,
        request: &MessageProofAnchorSubmissionRequest,
    ) -> Result<MessageProofAnchorSubmissionOutcome, MessageProofAnchoringError> {
        let runtime_request = self.runtime_request_for_anchor(request)?;
        let outcome = self
            .client
            .submit_commit(&runtime_request)
            .map_err(Self::map_runtime_commit_error)?;
        match outcome {
            KolmeRuntimeCommitOutcome::Submitted(receipt) => Ok(
                MessageProofAnchorSubmissionOutcome::Submitted(MessageProofAnchorReceipt {
                    provider: receipt.provider,
                    transaction_id: receipt.commit_id,
                }),
            ),
            KolmeRuntimeCommitOutcome::Duplicate(receipt) => Ok(
                MessageProofAnchorSubmissionOutcome::Duplicate(MessageProofAnchorReceipt {
                    provider: receipt.provider,
                    transaction_id: receipt.commit_id,
                }),
            ),
            KolmeRuntimeCommitOutcome::Rejected { reason } => {
                Ok(MessageProofAnchorSubmissionOutcome::Rejected { reason })
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Message proof anchoring service with deterministic retry/finality contracts.
pub struct MessageProofAnchoringService {
    submission_keys_by_message_id: BTreeMap<String, String>,
    finality_by_message_id: BTreeMap<String, MessageProofAnchorFinalityRecord>,
}

impl MessageProofAnchoringService {
    /// Creates an empty anchoring service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes deterministic idempotency key for one anchor request.
    pub fn idempotency_key_for_anchor(
        &self,
        request: &MessageProofAnchorRequest,
    ) -> Result<String, MessageProofAnchoringError> {
        Self::validate_anchor_request(request)?;
        Ok(format!(
            "message-anchor:{}:{}:{}:{}",
            request.message_id, request.actor_did, request.nonce, request.proof_hash
        ))
    }

    /// Anchors one message proof via chain adapter while enforcing lifecycle alignment.
    pub fn anchor_message_proof_via_chain_adapter<A: MessageProofChainAdapter>(
        &mut self,
        lifecycle: &mut MessageLifecycleStore,
        adapter: &mut A,
        request: MessageProofAnchorRequest,
    ) -> Result<MessageProofAnchorResult, MessageProofAnchoringError> {
        let status = lifecycle
            .status(request.message_id.as_str())
            .map_err(MessageProofAnchoringError::Lifecycle)?;
        if status != MessageStatus::Broadcast && status != MessageStatus::Included {
            return Err(MessageProofAnchoringError::InvalidAnchorState { found: status });
        }

        let idempotency_key = self.idempotency_key_for_anchor(&request)?;
        let retry_class =
            self.classify_retry_by_key(request.message_id.as_str(), idempotency_key.as_str());
        if retry_class == MessageProofAnchorRetryClass::ConflictNoRetry {
            let existing_key = self
                .submission_keys_by_message_id
                .get(request.message_id.as_str())
                .cloned()
                .unwrap_or_default();
            return Err(
                MessageProofAnchoringError::ConflictingAnchorIdempotencyKey {
                    message_id: request.message_id,
                    existing_key,
                    provided_key: idempotency_key,
                },
            );
        }

        if retry_class == MessageProofAnchorRetryClass::NewSubmission {
            self.submission_keys_by_message_id
                .insert(request.message_id.clone(), idempotency_key.clone());
        }

        let outcome = if retry_class == MessageProofAnchorRetryClass::FinalizedNoRetry {
            MessageProofAnchorSubmissionOutcome::FinalizedNoOp
        } else {
            adapter.submit_anchor(&MessageProofAnchorSubmissionRequest {
                message_id: request.message_id.clone(),
                actor_did: request.actor_did,
                nonce: request.nonce,
                proof_hash: request.proof_hash,
                idempotency_key: idempotency_key.clone(),
            })?
        };

        if matches!(
            outcome,
            MessageProofAnchorSubmissionOutcome::Submitted(_)
                | MessageProofAnchorSubmissionOutcome::Duplicate(_)
                | MessageProofAnchorSubmissionOutcome::FinalizedNoOp
        ) {
            let latest_status = lifecycle
                .status(request.message_id.as_str())
                .map_err(MessageProofAnchoringError::Lifecycle)?;
            if latest_status == MessageStatus::Broadcast {
                lifecycle
                    .transition(request.message_id.as_str(), MessageStatus::Included)
                    .map_err(MessageProofAnchoringError::Lifecycle)?;
            }
        }

        Ok(MessageProofAnchorResult {
            message_id: request.message_id,
            idempotency_key,
            retry_class,
            outcome,
        })
    }

    /// Records finality update for prior anchor submission.
    pub fn record_anchor_finality(
        &mut self,
        message_id: &str,
        idempotency_key: &str,
        sequence: u64,
        status: MessageProofAnchorFinalityStatus,
        receipt: &str,
    ) -> Result<(), MessageProofAnchoringError> {
        let Some(expected_key) = self.submission_keys_by_message_id.get(message_id) else {
            return Err(MessageProofAnchoringError::UnknownAnchorIdempotencyKey {
                message_id: message_id.to_owned(),
                idempotency_key: idempotency_key.to_owned(),
            });
        };
        if expected_key != idempotency_key {
            return Err(MessageProofAnchoringError::UnknownAnchorIdempotencyKey {
                message_id: message_id.to_owned(),
                idempotency_key: idempotency_key.to_owned(),
            });
        }

        if let Some(current) = self.finality_by_message_id.get(message_id) {
            if sequence < current.sequence {
                return Err(MessageProofAnchoringError::StaleFinalityUpdate {
                    message_id: message_id.to_owned(),
                    current_sequence: current.sequence,
                    attempted_sequence: sequence,
                });
            }

            if sequence == current.sequence {
                if current.idempotency_key == idempotency_key
                    && current.status == status
                    && current.receipt == receipt
                {
                    return Ok(());
                }

                return Err(MessageProofAnchoringError::ConflictingFinalityUpdate {
                    message_id: message_id.to_owned(),
                    sequence,
                });
            }

            if current.idempotency_key != idempotency_key {
                return Err(MessageProofAnchoringError::ConflictingFinalityUpdate {
                    message_id: message_id.to_owned(),
                    sequence,
                });
            }
        }

        self.finality_by_message_id.insert(
            message_id.to_owned(),
            MessageProofAnchorFinalityRecord {
                idempotency_key: idempotency_key.to_owned(),
                sequence,
                status,
                receipt: receipt.to_owned(),
            },
        );
        Ok(())
    }

    /// Returns most recent anchor finality record for message id, if present.
    pub fn anchor_finality(&self, message_id: &str) -> Option<&MessageProofAnchorFinalityRecord> {
        self.finality_by_message_id.get(message_id)
    }

    fn classify_retry_by_key(
        &self,
        message_id: &str,
        idempotency_key: &str,
    ) -> MessageProofAnchorRetryClass {
        let Some(existing_key) = self.submission_keys_by_message_id.get(message_id) else {
            return MessageProofAnchorRetryClass::NewSubmission;
        };

        if existing_key != idempotency_key {
            return MessageProofAnchorRetryClass::ConflictNoRetry;
        }

        if self.finality_by_message_id.contains_key(message_id) {
            return MessageProofAnchorRetryClass::FinalizedNoRetry;
        }

        MessageProofAnchorRetryClass::RetryableInFlight
    }

    fn validate_anchor_request(
        request: &MessageProofAnchorRequest,
    ) -> Result<(), MessageProofAnchoringError> {
        if request.message_id.trim().is_empty() {
            return Err(MessageProofAnchoringError::EmptyMessageId);
        }
        if request.actor_did.trim().is_empty() {
            return Err(MessageProofAnchoringError::EmptyActorDid);
        }
        parse_agent_did(
            request.actor_did.as_str(),
            "actor_did",
            MESSAGE_PROOF_ANCHOR_INVALID_ACTOR_DID_REASON_CODE,
        )?;
        if request.nonce == 0 {
            return Err(MessageProofAnchoringError::InvalidAnchorNonce(
                request.nonce,
            ));
        }
        if request.proof_hash.trim().is_empty() {
            return Err(MessageProofAnchoringError::EmptyProofHash);
        }
        Ok(())
    }
}

fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, MessageProofAnchoringError> {
    AgentDid::parse(value).map_err(|error| MessageProofAnchoringError::InvalidActorDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}
