use super::*;

pub(super) fn funding<'a>(
    store: &'a ServiceApiMessageStore,
    actor: &str,
    input: &FundInput,
) -> Result<Option<&'a ServiceApiPersistedEscrowRecord>, EscrowLifecycleError> {
    let existing = store.snapshot.escrows.values().find(|escrow| {
        escrow.funder_did.as_deref() == Some(actor)
            && escrow.fund_idempotency_key.as_deref() == Some(input.idempotency_key.as_str())
    });
    let Some(existing) = existing else {
        return Ok(None);
    };
    if funding_matches(existing, input) {
        return Ok(Some(existing));
    }
    Err(conflict(
        "ESCROW_IDEMPOTENCY_CONFLICT",
        "funding idempotency key was reused",
    ))
}

pub(super) fn release<'a>(
    store: &'a ServiceApiMessageStore,
    actor: &str,
    escrow_id: &str,
    key: &str,
) -> Result<Option<&'a ServiceApiEscrowTransitionReceiptRecord>, EscrowLifecycleError> {
    let receipt = store
        .snapshot
        .escrow_transition_receipts
        .iter()
        .find(|receipt| receipt.actor_did == actor && receipt.idempotency_key == key);
    let Some(receipt) = receipt else {
        return Ok(None);
    };
    if receipt.escrow_id == escrow_id && receipt.action == "escrow:release-authorize" {
        return Ok(Some(receipt));
    }
    Err(conflict(
        "ESCROW_IDEMPOTENCY_CONFLICT",
        "release idempotency key was reused",
    ))
}

fn funding_matches(record: &ServiceApiPersistedEscrowRecord, input: &FundInput) -> bool {
    record.task_id.as_deref() == Some(input.task_id.as_str())
        && record.transaction_id.as_deref() == Some(input.transaction_id.as_str())
        && record.beneficiary_did.as_deref() == Some(input.beneficiary_did.as_str())
        && record.amount_lamports == Some(input.amount_lamports)
        && record.network.as_deref() == Some(input.network.as_str())
        && record.terms_digest.as_deref() == Some(input.terms_digest.as_str())
        && record.release_authority_did.as_deref() == Some(input.release_authority_did.as_str())
        && record.release_policy.as_deref() == Some(input.release_policy.as_str())
}
