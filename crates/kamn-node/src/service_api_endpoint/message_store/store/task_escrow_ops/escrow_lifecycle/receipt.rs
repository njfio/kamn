use super::*;

pub(super) fn append(
    store: &mut ServiceApiMessageStore,
    record: &ServiceApiPersistedEscrowRecord,
    input: ReceiptInput<'_>,
) -> Result<ServiceApiEscrowTransitionReceiptRecord, EscrowLifecycleError> {
    let receipt_id = format!(
        "escrow-transition-receipt-{:08}",
        store.snapshot.escrow_transition_receipts.len() + 1
    );
    let receipt = build(record, receipt_id, input)?;
    store
        .snapshot
        .escrow_transition_receipts
        .push(receipt.clone());
    Ok(receipt)
}

pub(super) struct ReceiptInput<'a> {
    pub(super) actor: &'a str,
    pub(super) action: &'a str,
    pub(super) prior_state: &'a str,
    pub(super) key: String,
    pub(super) correlation_id: &'a str,
}

fn build(
    record: &ServiceApiPersistedEscrowRecord,
    receipt_id: String,
    input: ReceiptInput<'_>,
) -> Result<ServiceApiEscrowTransitionReceiptRecord, EscrowLifecycleError> {
    Ok(ServiceApiEscrowTransitionReceiptRecord {
        receipt_id,
        correlation_id: input.correlation_id.to_owned(),
        idempotency_key: input.key,
        actor_did: input.actor.to_owned(),
        escrow_id: record.escrow_id.clone(),
        task_id: required(&record.task_id)?,
        transaction_id: required(&record.transaction_id)?,
        action: input.action.to_owned(),
        prior_state: input.prior_state.to_owned(),
        resulting_state: record.state.clone(),
        network: required(&record.network)?,
        amount_lamports: record.amount_lamports.ok_or_else(migration_required)?,
        terms_digest: required(&record.terms_digest)?,
        release_policy: required(&record.release_policy)?,
    })
}

pub(super) fn response(
    record: &ServiceApiPersistedEscrowRecord,
    receipt: Option<&ServiceApiEscrowTransitionReceiptRecord>,
) -> ServiceApiEscrowStatusBody {
    receipt.map_or_else(
        || escrow_status_response(record),
        |value| receipt_status_response(record, value),
    )
}

fn required(value: &Option<String>) -> Result<String, EscrowLifecycleError> {
    value.clone().ok_or_else(migration_required)
}
