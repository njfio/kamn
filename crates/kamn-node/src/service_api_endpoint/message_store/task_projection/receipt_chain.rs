use super::*;
use crate::service_api_endpoint::projection_models::ServiceApiParticipantReceiptProjection;

mod binding;
mod commitment;
mod entry;
mod settlement;
mod validation;

pub(super) struct ReceiptChain {
    entries: Vec<ReceiptChainEntry>,
    pub(super) commitment: String,
}

pub(super) struct ReceiptChainEntry {
    receipt_id: String,
    receipt_digest: String,
    authorization_digest: String,
    actor_did: String,
    action: String,
    resource_id: String,
    correlation_id: String,
    idempotency_key: String,
    prior_state: String,
    resulting_state: String,
}

impl ReceiptChain {
    pub(super) fn participant_receipts(
        &self,
        requester: &str,
    ) -> Vec<ServiceApiParticipantReceiptProjection> {
        self.entries
            .iter()
            .filter(|entry| entry.actor_did == requester)
            .map(ReceiptChainEntry::participant_projection)
            .collect()
    }

    pub(super) fn task_receipt_ids(&self, requester: &str) -> Vec<String> {
        self.entries
            .iter()
            .filter(|entry| entry.actor_did == requester && entry.action.starts_with("task:"))
            .map(|entry| entry.receipt_id.clone())
            .collect()
    }
}

impl ReceiptChainEntry {
    fn participant_projection(&self) -> ServiceApiParticipantReceiptProjection {
        ServiceApiParticipantReceiptProjection {
            receipt_id: self.receipt_id.clone(),
            receipt_digest: self.receipt_digest.clone(),
            action: self.action.clone(),
            resource_id: self.resource_id.clone(),
            resulting_state: self.resulting_state.clone(),
        }
    }
}

pub(super) fn derive(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    task: &ServiceApiPersistedTaskRecord,
    escrow: &ServiceApiPersistedEscrowRecord,
) -> Result<ReceiptChain, TaskProjectionError> {
    let task_receipts = relevant_task_receipts(snapshot, task);
    let escrow_receipts = relevant_escrow_receipts(snapshot, escrow);
    validation::task_phases(task, &task_receipts)?;
    validation::escrow_phases(escrow, &escrow_receipts)?;
    let mut entries =
        entry::mutation_entries(snapshot, task, escrow, &task_receipts, &escrow_receipts)?;
    validation::unique_fields(&entries)?;
    settlement::append(snapshot, escrow, &mut entries)?;
    let commitment = commitment::chain(&entries);
    Ok(ReceiptChain {
        entries,
        commitment,
    })
}

fn relevant_task_receipts<'a>(
    snapshot: &'a ServiceApiPersistedMessageStoreSnapshot,
    task: &ServiceApiPersistedTaskRecord,
) -> Vec<&'a ServiceApiTaskTransitionReceiptRecord> {
    snapshot
        .task_transition_receipts
        .iter()
        .filter(|receipt| receipt.task_id == task.task_id)
        .collect()
}

fn relevant_escrow_receipts<'a>(
    snapshot: &'a ServiceApiPersistedMessageStoreSnapshot,
    escrow: &ServiceApiPersistedEscrowRecord,
) -> Vec<&'a ServiceApiEscrowTransitionReceiptRecord> {
    snapshot
        .escrow_transition_receipts
        .iter()
        .filter(|receipt| receipt.escrow_id == escrow.escrow_id)
        .collect()
}
