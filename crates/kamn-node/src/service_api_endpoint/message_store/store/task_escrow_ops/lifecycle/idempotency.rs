use super::input::{CreateInput, TransitionInput};
use super::*;

pub(super) fn find_create_retry<'a>(
    store: &'a ServiceApiMessageStore,
    actor: &str,
    input: &CreateInput,
) -> Result<Option<&'a ServiceApiPersistedTaskRecord>, TaskLifecycleError> {
    let existing = store.snapshot.tasks.values().find(|task| {
        task.creator_did.as_deref() == Some(actor)
            && task.create_idempotency_key.as_deref() == Some(input.idempotency_key.as_str())
    });
    let Some(existing) = existing else {
        return Ok(None);
    };
    if same_agreement(existing, input) {
        return Ok(Some(existing));
    }
    Err(conflict(
        "TASK_IDEMPOTENCY_CONFLICT",
        "creation idempotency key was reused",
    ))
}

pub(super) fn find_transition_retry<'a>(
    store: &'a ServiceApiMessageStore,
    actor: &str,
    task_id: &str,
    action: &str,
    input: &TransitionInput,
) -> Result<Option<&'a ServiceApiTaskTransitionReceiptRecord>, TaskLifecycleError> {
    let receipt = store
        .snapshot
        .task_transition_receipts
        .iter()
        .find(|receipt| {
            receipt.actor_did == actor && receipt.idempotency_key == input.idempotency_key
        });
    let Some(receipt) = receipt else {
        return Ok(None);
    };
    if retry_matches(receipt, task_id, action, input) {
        return Ok(Some(receipt));
    }
    Err(conflict(
        "TASK_IDEMPOTENCY_CONFLICT",
        "transition idempotency key was reused",
    ))
}

fn same_agreement(record: &ServiceApiPersistedTaskRecord, input: &CreateInput) -> bool {
    record.provider_did.as_deref() == Some(input.provider_did.as_str())
        && record.transaction_id.as_deref() == Some(input.transaction_id.as_str())
        && record.terms_digest.as_deref() == Some(input.terms_digest.as_str())
        && record.task_type == input.task_type
        && record.description == input.description
}

fn retry_matches(
    receipt: &ServiceApiTaskTransitionReceiptRecord,
    task_id: &str,
    action: &str,
    input: &TransitionInput,
) -> bool {
    receipt.task_id == task_id
        && receipt.action == action
        && receipt.completion_evidence_digest == input.completion_evidence_digest
}
