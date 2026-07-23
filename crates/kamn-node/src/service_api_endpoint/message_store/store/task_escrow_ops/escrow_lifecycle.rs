use super::super::super::*;
use super::settlement::{escrow_status_response, next_escrow_id, receipt_status_response};

mod agreement;
mod receipt;
mod retry;

use agreement::{
    build_record, issue_release_grant, parse_fund, parse_release_key, validate_funding,
    validate_release,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EscrowLifecycleError {
    BadRequest(&'static str, String),
    Forbidden(&'static str, String),
    Conflict(&'static str, String),
    NotFound,
    Persistence(String),
}

#[derive(Debug, Deserialize)]
pub(super) struct FundInput {
    pub(super) task_id: String,
    pub(super) transaction_id: String,
    pub(super) beneficiary_did: String,
    pub(super) amount_lamports: u64,
    pub(super) network: String,
    pub(super) terms_digest: String,
    pub(super) release_authority_did: String,
    pub(super) release_policy: String,
    pub(super) idempotency_key: String,
}

pub(super) fn fund(
    store: &mut ServiceApiMessageStore,
    actor: &str,
    payload: &str,
    correlation_id: &str,
) -> Result<ServiceApiEscrowStatusBody, EscrowLifecycleError> {
    store.refresh_from_disk().map_err(persistence)?;
    let input = parse_fund(payload)?;
    if let Some(existing) = retry::funding(store, actor, &input)? {
        return Ok(receipt::response(
            existing,
            Some(funding_receipt(store, existing)?),
        ));
    }
    let task = store
        .snapshot
        .tasks
        .get(&input.task_id)
        .cloned()
        .ok_or(EscrowLifecycleError::NotFound)?;
    validate_funding(actor, &input, &task)?;
    let escrow_id = next_escrow_id(store, payload);
    let key = input.idempotency_key.clone();
    let record = build_record(escrow_id.as_str(), actor, input);
    let receipt = receipt::append(
        store,
        &record,
        receipt::ReceiptInput {
            actor,
            action: "escrow:fund",
            prior_state: "unfunded",
            key,
            correlation_id,
        },
    )?;
    store.snapshot.escrows.insert(escrow_id.clone(), record);
    issue_release_grant(store, escrow_id.as_str(), actor);
    store.persist().map_err(persistence)?;
    Ok(receipt::response(
        &store.snapshot.escrows[&escrow_id],
        Some(&receipt),
    ))
}

pub(super) fn authorize_release(
    store: &mut ServiceApiMessageStore,
    actor: &str,
    escrow_id: &str,
    payload: &str,
    correlation_id: &str,
) -> Result<ServiceApiEscrowStatusBody, EscrowLifecycleError> {
    store.refresh_from_disk().map_err(persistence)?;
    let escrow = store
        .snapshot
        .escrows
        .get(escrow_id)
        .cloned()
        .ok_or(EscrowLifecycleError::NotFound)?;
    let key = parse_release_key(payload)?;
    if let Some(existing) = retry::release(store, actor, escrow_id, key.as_str())? {
        let record = &store.snapshot.escrows[escrow_id];
        return Ok(receipt::response(record, Some(existing)));
    }
    validate_release(store, actor, &escrow)?;
    let mut updated = escrow.clone();
    updated.state = "release-authorized".to_owned();
    let receipt = receipt::append(
        store,
        &updated,
        receipt::ReceiptInput {
            actor,
            action: "escrow:release-authorize",
            prior_state: "funded",
            key,
            correlation_id,
        },
    )?;
    store.snapshot.escrows.insert(escrow_id.to_owned(), updated);
    let response = receipt::response(&store.snapshot.escrows[escrow_id], Some(&receipt));
    store.persist().map_err(persistence)?;
    Ok(response)
}

pub(super) fn validate_release_eligibility(
    store: &mut ServiceApiMessageStore,
    actor: &str,
    escrow_id: &str,
) -> Result<(), EscrowLifecycleError> {
    store.refresh_from_disk().map_err(persistence)?;
    let escrow = store
        .snapshot
        .escrows
        .get(escrow_id)
        .cloned()
        .ok_or(EscrowLifecycleError::NotFound)?;
    validate_release(store, actor, &escrow)
}

fn funding_receipt<'a>(
    store: &'a ServiceApiMessageStore,
    escrow: &ServiceApiPersistedEscrowRecord,
) -> Result<&'a ServiceApiEscrowTransitionReceiptRecord, EscrowLifecycleError> {
    store
        .snapshot
        .escrow_transition_receipts
        .iter()
        .find(|receipt| receipt.escrow_id == escrow.escrow_id && receipt.action == "escrow:fund")
        .ok_or_else(|| {
            conflict(
                "ESCROW_RECEIPT_MISSING",
                "escrow funding receipt is missing",
            )
        })
}

fn migration_required() -> EscrowLifecycleError {
    conflict(
        "ESCROW_TASK_AGREEMENT_REQUIRED",
        "escrow lacks canonical task agreement",
    )
}
fn bad(code: &'static str, message: impl Into<String>) -> EscrowLifecycleError {
    EscrowLifecycleError::BadRequest(code, message.into())
}
fn forbidden(code: &'static str, message: impl Into<String>) -> EscrowLifecycleError {
    EscrowLifecycleError::Forbidden(code, message.into())
}
fn conflict(code: &'static str, message: impl Into<String>) -> EscrowLifecycleError {
    EscrowLifecycleError::Conflict(code, message.into())
}
fn persistence(message: String) -> EscrowLifecycleError {
    EscrowLifecycleError::Persistence(message)
}
