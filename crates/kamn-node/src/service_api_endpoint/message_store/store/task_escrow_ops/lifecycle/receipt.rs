use super::agreement::required_agreement;
use super::*;

pub(super) struct ReceiptInput<'a> {
    pub(super) actor: &'a str,
    pub(super) action: &'a str,
    pub(super) prior_state: String,
    pub(super) idempotency_key: String,
    pub(super) correlation_id: &'a str,
    pub(super) sequence: usize,
}

pub(super) fn build(
    record: &ServiceApiPersistedTaskRecord,
    input: ReceiptInput<'_>,
) -> Result<ServiceApiTaskTransitionReceiptRecord, TaskLifecycleError> {
    Ok(ServiceApiTaskTransitionReceiptRecord {
        receipt_id: format!("task-transition-receipt-{:08}", input.sequence),
        correlation_id: input.correlation_id.to_owned(),
        idempotency_key: input.idempotency_key,
        actor_did: input.actor.to_owned(),
        task_id: record.task_id.clone(),
        transaction_id: required_agreement(&record.transaction_id)?,
        action: input.action.to_owned(),
        prior_state: input.prior_state,
        resulting_state: record.state.clone(),
        terms_digest: required_agreement(&record.terms_digest)?,
        completion_evidence_digest: record.completion_evidence_digest.clone(),
    })
}

pub(super) fn create_response(
    record: &ServiceApiPersistedTaskRecord,
    receipt: &ServiceApiTaskTransitionReceiptRecord,
) -> ServiceApiTaskCreateBody {
    ServiceApiTaskCreateBody {
        task_id: record.task_id.clone(),
        state: receipt.resulting_state.clone(),
        transaction_id: record.transaction_id.clone(),
        creator_did: record.creator_did.clone(),
        provider_did: record.provider_did.clone(),
        terms_digest: record.terms_digest.clone(),
        receipt_id: receipt.receipt_id.clone(),
        receipt_digest: authority_digest::task(receipt),
        action: receipt.action.clone(),
    }
}

pub(super) fn transition_response(
    record: &ServiceApiPersistedTaskRecord,
    receipt: &ServiceApiTaskTransitionReceiptRecord,
) -> ServiceApiTaskTransitionBody {
    ServiceApiTaskTransitionBody {
        task_id: record.task_id.clone(),
        state: record.state.clone(),
        transaction_id: record.transaction_id.clone(),
        creator_did: record.creator_did.clone(),
        provider_did: record.provider_did.clone(),
        terms_digest: record.terms_digest.clone(),
        receipt_id: Some(receipt.receipt_id.clone()),
        receipt_digest: Some(authority_digest::task(receipt)),
        action: receipt.action.clone(),
    }
}

pub(super) fn retry_response(
    store: &ServiceApiMessageStore,
    receipt: &ServiceApiTaskTransitionReceiptRecord,
) -> ServiceApiTaskTransitionBody {
    let mut response = transition_response(&store.snapshot.tasks[&receipt.task_id], receipt);
    response.state = receipt.resulting_state.clone();
    response
}
