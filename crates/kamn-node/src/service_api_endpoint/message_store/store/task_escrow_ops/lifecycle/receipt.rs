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

pub(super) fn create_response(record: &ServiceApiPersistedTaskRecord) -> ServiceApiTaskCreateBody {
    ServiceApiTaskCreateBody {
        task_id: record.task_id.clone(),
        state: record.state.clone(),
        transaction_id: record.transaction_id.clone(),
        creator_did: record.creator_did.clone(),
        provider_did: record.provider_did.clone(),
        terms_digest: record.terms_digest.clone(),
    }
}

pub(super) fn transition_response(
    record: &ServiceApiPersistedTaskRecord,
    receipt_id: &str,
) -> ServiceApiTaskTransitionBody {
    ServiceApiTaskTransitionBody {
        task_id: record.task_id.clone(),
        state: record.state.clone(),
        transaction_id: record.transaction_id.clone(),
        creator_did: record.creator_did.clone(),
        provider_did: record.provider_did.clone(),
        terms_digest: record.terms_digest.clone(),
        receipt_id: Some(receipt_id.to_owned()),
    }
}

pub(super) fn retry_response(
    store: &ServiceApiMessageStore,
    receipt: &ServiceApiTaskTransitionReceiptRecord,
) -> ServiceApiTaskTransitionBody {
    transition_response(&store.snapshot.tasks[&receipt.task_id], &receipt.receipt_id)
}
