//! Runtime commit lifecycle pipeline state transitions and projections.

use super::{
    is_kolme_valid_receipt_commit_id_input_contract,
    is_kolme_valid_receipt_provider_input_contract, lifecycle_state_for_finality_contract,
    lifecycle_state_label_contract, KolmeCommitReceiptFinality, KolmeRuntimeCommitClient,
    KolmeRuntimeCommitError, KolmeRuntimeCommitOutcome, KolmeRuntimeCommitRequest,
    RuntimeCommitLifecycleState,
};
use std::collections::HashMap;

/// One runtime operation lifecycle record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeCommitLifecycleRecord {
    /// Runtime operation identifier.
    pub operation_id: String,
    /// Deterministic idempotency key for the operation.
    pub idempotency_key: String,
    /// Projected lifecycle state.
    pub state: RuntimeCommitLifecycleState,
    /// Whether runtime should requeue/retry polling for this operation.
    pub needs_requeue: bool,
    /// Last known receipt provider marker.
    pub receipt_provider: Option<String>,
    /// Last known receipt identifier.
    pub receipt_commit_id: Option<String>,
    /// Last known rejection/failure reason.
    pub last_error_reason: Option<String>,
}

/// Projection summary for runtime commit lifecycle counts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeCommitFinalityProjection {
    /// Number of pending operations.
    pub pending_count: usize,
    /// Number of finalized operations.
    pub final_count: usize,
    /// Number of failed operations.
    pub failed_count: usize,
}

/// Deterministic runtime pipeline for commit receipt confirmation and finality projection.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeCommitPipeline {
    records_by_operation_id: HashMap<String, RuntimeCommitLifecycleRecord>,
}

impl RuntimeCommitPipeline {
    /// Constructs an empty runtime commit pipeline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Submits one runtime commit through the provided commit client and records lifecycle state.
    pub fn submit_with_client<C: KolmeRuntimeCommitClient>(
        &mut self,
        client: &mut C,
        request: KolmeRuntimeCommitRequest,
    ) -> Result<RuntimeCommitLifecycleRecord, KolmeRuntimeCommitError> {
        let outcome = client.submit_commit(&request)?;
        let record = match &outcome {
            KolmeRuntimeCommitOutcome::Submitted(receipt)
            | KolmeRuntimeCommitOutcome::Duplicate(receipt) => {
                let state = lifecycle_state_for_finality_contract(receipt.finality);
                RuntimeCommitLifecycleRecord {
                    operation_id: request.operation_id.clone(),
                    idempotency_key: request.idempotency_key().to_owned(),
                    state,
                    needs_requeue: matches!(state, RuntimeCommitLifecycleState::Pending),
                    receipt_provider: Some(receipt.provider.clone()),
                    receipt_commit_id: Some(receipt.commit_id.clone()),
                    last_error_reason: None,
                }
            }
            KolmeRuntimeCommitOutcome::Rejected { reason } => RuntimeCommitLifecycleRecord {
                operation_id: request.operation_id.clone(),
                idempotency_key: request.idempotency_key().to_owned(),
                state: RuntimeCommitLifecycleState::Failed,
                needs_requeue: false,
                receipt_provider: None,
                receipt_commit_id: None,
                last_error_reason: Some(reason.clone()),
            },
        };
        self.records_by_operation_id
            .insert(request.operation_id.clone(), record.clone());
        Ok(record)
    }

    /// Applies explicit receipt finality update for an existing operation.
    pub fn apply_receipt_finality(
        &mut self,
        operation_id: &str,
        finality: KolmeCommitReceiptFinality,
        receipt_provider: &str,
        receipt_commit_id: &str,
    ) -> Result<RuntimeCommitLifecycleRecord, KolmeRuntimeCommitError> {
        if !is_kolme_valid_receipt_provider_input_contract(receipt_provider) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "receipt_provider",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_receipt_commit_id_input_contract(receipt_commit_id) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "receipt_commit_id",
                reason: "must not be empty",
            });
        }

        let record = self.records_by_operation_id.get_mut(operation_id).ok_or(
            KolmeRuntimeCommitError::UnknownOperationId {
                operation_id: operation_id.to_owned(),
            },
        )?;

        if let Some(expected_provider) = record.receipt_provider.as_deref() {
            if expected_provider != receipt_provider {
                return Err(KolmeRuntimeCommitError::ReceiptFieldMismatch {
                    field: "receipt_provider",
                    expected: expected_provider.to_owned(),
                    observed: receipt_provider.to_owned(),
                });
            }
        }
        if let Some(expected_commit_id) = record.receipt_commit_id.as_deref() {
            if expected_commit_id != receipt_commit_id {
                return Err(KolmeRuntimeCommitError::ReceiptFieldMismatch {
                    field: "receipt_commit_id",
                    expected: expected_commit_id.to_owned(),
                    observed: receipt_commit_id.to_owned(),
                });
            }
        }

        let target_state = lifecycle_state_for_finality_contract(finality);

        if record.state != target_state
            && !matches!(
                (record.state, target_state),
                (
                    RuntimeCommitLifecycleState::Pending,
                    RuntimeCommitLifecycleState::Finalized
                ) | (
                    RuntimeCommitLifecycleState::Pending,
                    RuntimeCommitLifecycleState::Failed
                )
            )
        {
            return Err(KolmeRuntimeCommitError::InvalidFinalityTransition {
                from: lifecycle_state_label_contract(record.state),
                to: lifecycle_state_label_contract(target_state),
            });
        }

        record.state = target_state;
        record.needs_requeue = matches!(target_state, RuntimeCommitLifecycleState::Pending);
        record.receipt_provider = Some(receipt_provider.to_owned());
        record.receipt_commit_id = Some(receipt_commit_id.to_owned());
        if !matches!(target_state, RuntimeCommitLifecycleState::Failed) {
            record.last_error_reason = None;
        }
        Ok(record.clone())
    }

    /// Returns lifecycle record for the provided runtime operation identifier.
    pub fn record(&self, operation_id: &str) -> Option<&RuntimeCommitLifecycleRecord> {
        self.records_by_operation_id.get(operation_id)
    }

    /// Computes deterministic pending/final/failed projection counts.
    pub fn finality_projection(&self) -> RuntimeCommitFinalityProjection {
        let mut projection = RuntimeCommitFinalityProjection::default();
        for record in self.records_by_operation_id.values() {
            match record.state {
                RuntimeCommitLifecycleState::Pending => projection.pending_count += 1,
                RuntimeCommitLifecycleState::Finalized => projection.final_count += 1,
                RuntimeCommitLifecycleState::Failed => projection.failed_count += 1,
            }
        }
        projection
    }
}
