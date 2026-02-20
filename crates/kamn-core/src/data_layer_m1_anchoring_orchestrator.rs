//! M1 anchoring orchestrator contracts for deterministic scheduler-to-persistence planning.
//!
//! This module composes scheduler threshold evaluation, merkle batch assembly,
//! and anchoring outcomes into a deterministic persistence plan that adapter
//! layers can execute.

use crate::{
    evaluate_data_layer_m1_batch_trigger, DataLayerM1AnchorOutcome, DataLayerM1AnchorResult,
    DataLayerM1AnchorRetryClass, DataLayerM1BatchSchedulerError, DataLayerM1BatchSchedulerPolicy,
    DataLayerM1BatchTriggerDecision, DataLayerM1Error, DataLayerM1KolmeAnchoringWorker,
    DataLayerM1MerkleBatch, DataLayerM1MerkleLeaf, DataLayerM1PendingBatchMessage,
    KolmeCommitReceiptFinality,
};
use std::fmt;

/// Reason marker for deferred orchestrator tick outcomes.
pub const DATA_LAYER_M1_ANCHORING_TICK_DEFERRED_REASON_CODE: &str = "m1_anchoring_tick_deferred";
/// Reason marker for planned orchestrator tick outcomes.
pub const DATA_LAYER_M1_ANCHORING_TICK_PLANNED_REASON_CODE: &str = "m1_anchoring_tick_planned";
/// Reason marker for rejected orchestrator tick outcomes.
pub const DATA_LAYER_M1_ANCHORING_TICK_REJECTED_REASON_CODE: &str = "m1_anchoring_tick_rejected";
/// Reason marker for final-receipt paths missing confirmation metadata.
pub const DATA_LAYER_M1_ANCHORING_CONFIRMATION_HINT_REQUIRED_REASON_CODE: &str =
    "m1_anchoring_confirmation_hint_required_for_final_receipt";
/// Reason marker for retryable in-flight follow-up policy decisions.
pub const DATA_LAYER_M1_ANCHORING_FOLLOW_UP_RETRY_IN_FLIGHT_REASON_CODE: &str =
    "m1_anchoring_follow_up_retry_in_flight";
/// Reason marker for pending confirmation follow-up policy decisions.
pub const DATA_LAYER_M1_ANCHORING_FOLLOW_UP_POLL_PENDING_REASON_CODE: &str =
    "m1_anchoring_follow_up_poll_pending";
/// Reason marker for conflict/no-retry follow-up policy decisions.
pub const DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_CONFLICT_REASON_CODE: &str =
    "m1_anchoring_follow_up_no_retry_conflict";
/// Reason marker for finalized/no-retry follow-up policy decisions.
pub const DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_FINAL_REASON_CODE: &str =
    "m1_anchoring_follow_up_no_retry_final";
/// Reason marker for failed/no-retry follow-up policy decisions.
pub const DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_FAILED_REASON_CODE: &str =
    "m1_anchoring_follow_up_no_retry_failed";

const DATA_LAYER_M1_ANCHORING_RETRY_BACKOFF_SECONDS: u64 = 60;
const DATA_LAYER_M1_ANCHORING_CONFIRMATION_POLL_SECONDS: u64 = 30;

/// Deterministic follow-up action after one anchoring attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1AnchoringFollowUpAction {
    /// Retry anchoring after a deterministic delay.
    Retry,
    /// Poll provider confirmation after a deterministic delay.
    PollConfirmation,
    /// Do not retry/poll this batch automatically.
    NoRetry,
}

/// Deterministic follow-up policy projected from retry-class and receipt finality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchoringFollowUpPolicy {
    /// Follow-up action classification.
    pub action: DataLayerM1AnchoringFollowUpAction,
    /// Stable reason marker for this policy decision.
    pub reason_code: &'static str,
    /// Retry timestamp when action is `Retry`.
    pub retry_after_unix_seconds: Option<u64>,
    /// Poll timestamp when action is `PollConfirmation`.
    pub poll_after_unix_seconds: Option<u64>,
    /// Retry-class projection from anchoring worker result.
    pub retry_class: DataLayerM1AnchorRetryClass,
    /// Optional receipt finality when provider receipt exists.
    pub receipt_finality: Option<KolmeCommitReceiptFinality>,
}

/// One projected message assignment for merkle batch persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchoringMessageAssignment {
    /// Message identifier.
    pub message_id: String,
    /// Assigned leaf index.
    pub leaf_index: i32,
}

/// Submission metadata projected for merkle batch persistence updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchoringSubmissionMetadata {
    /// Provider transaction hash/identifier.
    pub kolme_tx_hash: String,
    /// Submission timestamp as unix seconds.
    pub submitted_at_unix_seconds: i64,
}

/// Confirmation metadata projected for merkle batch persistence updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchoringConfirmationMetadata {
    /// Confirmed block height.
    pub kolme_block_height: i64,
    /// Confirmation timestamp as unix seconds.
    pub confirmed_at_unix_seconds: i64,
}

/// Deterministic persistence-plan projection for one anchoring tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM1AnchoringPersistencePlan {
    /// Merkle batch identifier.
    pub batch_id: String,
    /// Merkle root hash.
    pub merkle_root: String,
    /// Leaf count in this batch.
    pub leaf_count: i32,
    /// Scheduler timestamp used to persist batch creation.
    pub scheduled_at_unix_seconds: i64,
    /// Message-to-leaf assignment list.
    pub assignments: Vec<DataLayerM1AnchoringMessageAssignment>,
    /// Submission metadata for `submitted` persistence updates.
    pub submission: Option<DataLayerM1AnchoringSubmissionMetadata>,
    /// Confirmation metadata for `confirmed` persistence updates.
    pub confirmation: Option<DataLayerM1AnchoringConfirmationMetadata>,
}

/// Deterministic tick outcome for one orchestrator pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1AnchoringTickOutcome {
    /// Scheduler deferred anchoring.
    Deferred {
        /// Stable reason code.
        reason_code: &'static str,
        /// Pending message count.
        pending_count: usize,
    },
    /// Anchoring succeeded and persistence metadata was projected.
    Planned {
        /// Stable reason code.
        reason_code: &'static str,
        /// Assembled merkle batch.
        batch: Box<DataLayerM1MerkleBatch>,
        /// Anchoring worker result.
        anchor_result: DataLayerM1AnchorResult,
        /// Deterministic persistence plan.
        persistence_plan: Box<DataLayerM1AnchoringPersistencePlan>,
        /// Deterministic follow-up policy projection for retry/confirmation flow.
        follow_up_policy: DataLayerM1AnchoringFollowUpPolicy,
    },
    /// Anchoring was rejected by provider.
    Rejected {
        /// Stable reason code.
        reason_code: &'static str,
        /// Assembled merkle batch that was rejected.
        batch: Box<DataLayerM1MerkleBatch>,
        /// Provider rejection reason.
        rejection_reason: String,
        /// Deterministic follow-up policy for this rejected outcome.
        follow_up_policy: DataLayerM1AnchoringFollowUpPolicy,
    },
}

/// Error taxonomy for M1 orchestrator planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM1AnchoringOrchestratorError {
    /// Invalid orchestrator input payload.
    InvalidInput {
        /// Invalid field name.
        field: &'static str,
        /// Validation detail.
        detail: String,
    },
    /// Scheduler evaluation failed fail-closed.
    Scheduler(DataLayerM1BatchSchedulerError),
    /// M1 merkle/anchoring contract failed fail-closed.
    M1(DataLayerM1Error),
    /// Final receipt needs explicit confirmation metadata.
    MissingConfirmationMetadata {
        /// Stable reason code.
        reason_code: &'static str,
        /// Provider transaction identifier.
        transaction_id: String,
    },
}

impl fmt::Display for DataLayerM1AnchoringOrchestratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput { field, detail } => {
                write!(formatter, "invalid orchestrator input {field}: {detail}")
            }
            Self::Scheduler(error) => write!(formatter, "scheduler evaluation failed: {error}"),
            Self::M1(error) => write!(formatter, "m1 evaluation failed: {error}"),
            Self::MissingConfirmationMetadata {
                reason_code,
                transaction_id,
            } => write!(
                formatter,
                "missing confirmation metadata for final receipt {transaction_id}: {reason_code}"
            ),
        }
    }
}

impl std::error::Error for DataLayerM1AnchoringOrchestratorError {}

impl From<DataLayerM1BatchSchedulerError> for DataLayerM1AnchoringOrchestratorError {
    fn from(value: DataLayerM1BatchSchedulerError) -> Self {
        Self::Scheduler(value)
    }
}

impl From<DataLayerM1Error> for DataLayerM1AnchoringOrchestratorError {
    fn from(value: DataLayerM1Error) -> Self {
        Self::M1(value)
    }
}

/// Deterministic M1 orchestrator that owns scheduler and anchoring worker state.
#[derive(Debug, Clone)]
pub struct DataLayerM1AnchoringOrchestrator<C> {
    scheduler_policy: DataLayerM1BatchSchedulerPolicy,
    worker: DataLayerM1KolmeAnchoringWorker<C>,
}

impl<C> DataLayerM1AnchoringOrchestrator<C> {
    /// Creates a new orchestrator with scheduler policy and worker state.
    pub fn new(
        client: C,
        actor_did: &str,
        state_root_prefix: &str,
        scheduler_policy: DataLayerM1BatchSchedulerPolicy,
    ) -> Result<Self, DataLayerM1AnchoringOrchestratorError> {
        let worker = DataLayerM1KolmeAnchoringWorker::new(client, actor_did, state_root_prefix)?;
        Ok(Self {
            scheduler_policy,
            worker,
        })
    }
}

impl<C> DataLayerM1AnchoringOrchestrator<C>
where
    C: crate::KolmeRuntimeCommitClient,
{
    /// Evaluates one orchestrator tick and projects deterministic persistence metadata.
    pub fn plan_tick(
        &mut self,
        pending_messages: &[DataLayerM1PendingBatchMessage],
        now_unix_seconds: u64,
        scheduled_at_unix_seconds: i64,
        submitted_at_unix_seconds: i64,
        confirmation_metadata: Option<DataLayerM1AnchoringConfirmationMetadata>,
    ) -> Result<DataLayerM1AnchoringTickOutcome, DataLayerM1AnchoringOrchestratorError> {
        validate_positive_timestamp(scheduled_at_unix_seconds, "scheduled_at_unix_seconds")?;
        validate_positive_timestamp(submitted_at_unix_seconds, "submitted_at_unix_seconds")?;

        let scheduler_decision = evaluate_data_layer_m1_batch_trigger(
            &self.scheduler_policy,
            pending_messages,
            now_unix_seconds,
        )?;
        if matches!(
            scheduler_decision,
            DataLayerM1BatchTriggerDecision::Deferred { .. }
        ) {
            return Ok(DataLayerM1AnchoringTickOutcome::Deferred {
                reason_code: DATA_LAYER_M1_ANCHORING_TICK_DEFERRED_REASON_CODE,
                pending_count: pending_messages.len(),
            });
        }

        let batch = assemble_batch_for_pending_messages(pending_messages)?;
        let anchor_result = self.worker.anchor_batch(&batch)?;
        let follow_up_policy = project_follow_up_policy(&anchor_result, now_unix_seconds);

        match &anchor_result.outcome {
            DataLayerM1AnchorOutcome::Rejected { reason } => {
                Ok(DataLayerM1AnchoringTickOutcome::Rejected {
                    reason_code: DATA_LAYER_M1_ANCHORING_TICK_REJECTED_REASON_CODE,
                    batch: Box::new(batch),
                    rejection_reason: reason.clone(),
                    follow_up_policy,
                })
            }
            DataLayerM1AnchorOutcome::Submitted(receipt)
            | DataLayerM1AnchorOutcome::Duplicate(receipt) => {
                let confirmation = match receipt.finality {
                    KolmeCommitReceiptFinality::Final => {
                        let confirmation = confirmation_metadata.ok_or(
                            DataLayerM1AnchoringOrchestratorError::MissingConfirmationMetadata {
                                reason_code:
                                    DATA_LAYER_M1_ANCHORING_CONFIRMATION_HINT_REQUIRED_REASON_CODE,
                                transaction_id: receipt.transaction_id.clone(),
                            },
                        )?;
                        validate_confirmation_metadata(&confirmation)?;
                        Some(confirmation)
                    }
                    _ => None,
                };
                let persistence_plan = DataLayerM1AnchoringPersistencePlan {
                    batch_id: batch.batch_id.clone(),
                    merkle_root: batch.merkle_root.clone(),
                    leaf_count: i32::try_from(batch.message_count).map_err(|error| {
                        DataLayerM1AnchoringOrchestratorError::InvalidInput {
                            field: "leaf_count",
                            detail: format!("message_count conversion failed: {error}"),
                        }
                    })?,
                    scheduled_at_unix_seconds,
                    assignments: project_assignments(&batch)?,
                    submission: Some(DataLayerM1AnchoringSubmissionMetadata {
                        kolme_tx_hash: receipt.transaction_id.clone(),
                        submitted_at_unix_seconds,
                    }),
                    confirmation,
                };
                Ok(DataLayerM1AnchoringTickOutcome::Planned {
                    reason_code: DATA_LAYER_M1_ANCHORING_TICK_PLANNED_REASON_CODE,
                    batch: Box::new(batch),
                    anchor_result,
                    persistence_plan: Box::new(persistence_plan),
                    follow_up_policy,
                })
            }
        }
    }
}

fn validate_positive_timestamp(
    value: i64,
    field: &'static str,
) -> Result<(), DataLayerM1AnchoringOrchestratorError> {
    if value <= 0 {
        return Err(DataLayerM1AnchoringOrchestratorError::InvalidInput {
            field,
            detail: "must be greater than zero".to_owned(),
        });
    }
    Ok(())
}

fn validate_confirmation_metadata(
    metadata: &DataLayerM1AnchoringConfirmationMetadata,
) -> Result<(), DataLayerM1AnchoringOrchestratorError> {
    if metadata.kolme_block_height < 0 {
        return Err(DataLayerM1AnchoringOrchestratorError::InvalidInput {
            field: "kolme_block_height",
            detail: "must be greater than or equal to zero".to_owned(),
        });
    }
    validate_positive_timestamp(
        metadata.confirmed_at_unix_seconds,
        "confirmed_at_unix_seconds",
    )
}

fn assemble_batch_for_pending_messages(
    pending_messages: &[DataLayerM1PendingBatchMessage],
) -> Result<DataLayerM1MerkleBatch, DataLayerM1AnchoringOrchestratorError> {
    let mut ordered = pending_messages.to_vec();
    ordered.sort_by(|left, right| {
        left.created_at_unix_seconds
            .cmp(&right.created_at_unix_seconds)
            .then(left.message_id.cmp(&right.message_id))
    });

    let mut leaves = Vec::with_capacity(ordered.len());
    for (index, pending) in ordered.iter().enumerate() {
        let leaf_index = u32::try_from(index).map_err(|error| {
            DataLayerM1AnchoringOrchestratorError::InvalidInput {
                field: "leaf_index",
                detail: format!("leaf index conversion failed: {error}"),
            }
        })?;
        leaves.push(DataLayerM1MerkleLeaf {
            message_id: pending.message_id.clone(),
            leaf_index,
            content_hash: pending.content_hash.clone(),
        });
    }

    Ok(DataLayerM1MerkleBatch::assemble(leaves)?)
}

fn project_assignments(
    batch: &DataLayerM1MerkleBatch,
) -> Result<Vec<DataLayerM1AnchoringMessageAssignment>, DataLayerM1AnchoringOrchestratorError> {
    batch
        .leaves()
        .iter()
        .map(|leaf| {
            let leaf_index = i32::try_from(leaf.leaf_index).map_err(|error| {
                DataLayerM1AnchoringOrchestratorError::InvalidInput {
                    field: "leaf_index",
                    detail: format!("leaf index conversion failed: {error}"),
                }
            })?;
            Ok(DataLayerM1AnchoringMessageAssignment {
                message_id: leaf.message_id.clone(),
                leaf_index,
            })
        })
        .collect()
}

fn project_follow_up_policy(
    anchor_result: &DataLayerM1AnchorResult,
    now_unix_seconds: u64,
) -> DataLayerM1AnchoringFollowUpPolicy {
    if anchor_result.retry_class == DataLayerM1AnchorRetryClass::RetryableInFlight {
        return DataLayerM1AnchoringFollowUpPolicy {
            action: DataLayerM1AnchoringFollowUpAction::Retry,
            reason_code: DATA_LAYER_M1_ANCHORING_FOLLOW_UP_RETRY_IN_FLIGHT_REASON_CODE,
            retry_after_unix_seconds: Some(
                now_unix_seconds.saturating_add(DATA_LAYER_M1_ANCHORING_RETRY_BACKOFF_SECONDS),
            ),
            poll_after_unix_seconds: None,
            retry_class: anchor_result.retry_class,
            receipt_finality: receipt_finality(anchor_result),
        };
    }

    match receipt_finality(anchor_result) {
        Some(KolmeCommitReceiptFinality::Pending) => DataLayerM1AnchoringFollowUpPolicy {
            action: DataLayerM1AnchoringFollowUpAction::PollConfirmation,
            reason_code: DATA_LAYER_M1_ANCHORING_FOLLOW_UP_POLL_PENDING_REASON_CODE,
            retry_after_unix_seconds: None,
            poll_after_unix_seconds: Some(
                now_unix_seconds.saturating_add(DATA_LAYER_M1_ANCHORING_CONFIRMATION_POLL_SECONDS),
            ),
            retry_class: anchor_result.retry_class,
            receipt_finality: Some(KolmeCommitReceiptFinality::Pending),
        },
        Some(KolmeCommitReceiptFinality::Final) => DataLayerM1AnchoringFollowUpPolicy {
            action: DataLayerM1AnchoringFollowUpAction::NoRetry,
            reason_code: DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_FINAL_REASON_CODE,
            retry_after_unix_seconds: None,
            poll_after_unix_seconds: None,
            retry_class: anchor_result.retry_class,
            receipt_finality: Some(KolmeCommitReceiptFinality::Final),
        },
        Some(KolmeCommitReceiptFinality::Failed) => DataLayerM1AnchoringFollowUpPolicy {
            action: DataLayerM1AnchoringFollowUpAction::NoRetry,
            reason_code: DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_FAILED_REASON_CODE,
            retry_after_unix_seconds: None,
            poll_after_unix_seconds: None,
            retry_class: anchor_result.retry_class,
            receipt_finality: Some(KolmeCommitReceiptFinality::Failed),
        },
        None => DataLayerM1AnchoringFollowUpPolicy {
            action: DataLayerM1AnchoringFollowUpAction::NoRetry,
            reason_code: DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_CONFLICT_REASON_CODE,
            retry_after_unix_seconds: None,
            poll_after_unix_seconds: None,
            retry_class: anchor_result.retry_class,
            receipt_finality: None,
        },
    }
}

fn receipt_finality(anchor_result: &DataLayerM1AnchorResult) -> Option<KolmeCommitReceiptFinality> {
    match &anchor_result.outcome {
        DataLayerM1AnchorOutcome::Submitted(receipt)
        | DataLayerM1AnchorOutcome::Duplicate(receipt) => Some(receipt.finality),
        DataLayerM1AnchorOutcome::Rejected { .. } => None,
    }
}
