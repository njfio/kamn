use super::*;

pub(super) fn append(
    store: &mut ServiceApiMessageStore,
    record: &ServiceApiPersistedEscrowRecord,
    actor: &str,
    action: &str,
    prior_state: &str,
    key: String,
    correlation_id: &str,
) -> Result<String, EscrowLifecycleError> {
    let receipt_id = format!(
        "escrow-transition-receipt-{:08}",
        store.snapshot.escrow_transition_receipts.len() + 1
    );
    store
        .snapshot
        .escrow_transition_receipts
        .push(ServiceApiEscrowTransitionReceiptRecord {
            receipt_id: receipt_id.clone(),
            correlation_id: correlation_id.to_owned(),
            idempotency_key: key,
            actor_did: actor.to_owned(),
            escrow_id: record.escrow_id.clone(),
            task_id: required(&record.task_id)?,
            transaction_id: required(&record.transaction_id)?,
            action: action.to_owned(),
            prior_state: prior_state.to_owned(),
            resulting_state: record.state.clone(),
            network: required(&record.network)?,
            amount_lamports: record.amount_lamports.ok_or_else(migration_required)?,
            terms_digest: required(&record.terms_digest)?,
            release_policy: required(&record.release_policy)?,
        });
    Ok(receipt_id)
}

pub(super) fn response(
    record: &ServiceApiPersistedEscrowRecord,
    receipt_id: Option<&str>,
) -> ServiceApiEscrowStatusBody {
    let mut response = escrow_status_response(record);
    response.receipt_id = receipt_id.map(str::to_owned);
    response
}

fn required(value: &Option<String>) -> Result<String, EscrowLifecycleError> {
    value.clone().ok_or_else(migration_required)
}
