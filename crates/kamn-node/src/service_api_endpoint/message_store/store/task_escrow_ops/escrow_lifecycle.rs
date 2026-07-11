use super::super::super::*;
use super::settlement::{escrow_status_response, next_escrow_id};

const MAX_AMOUNT_LAMPORTS: u64 = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EscrowLifecycleError {
    BadRequest(&'static str, String),
    Forbidden(&'static str, String),
    Conflict(&'static str, String),
    NotFound,
    Persistence(String),
}

#[derive(Debug, Deserialize)]
struct FundInput {
    task_id: String,
    transaction_id: String,
    beneficiary_did: String,
    amount_lamports: u64,
    network: String,
    terms_digest: String,
    release_authority_did: String,
    release_policy: String,
    idempotency_key: String,
}

pub(super) fn fund(
    store: &mut ServiceApiMessageStore,
    actor: &str,
    payload: &str,
) -> Result<ServiceApiEscrowStatusBody, EscrowLifecycleError> {
    store.refresh_from_disk().map_err(persistence)?;
    let input = parse_fund(payload)?;
    let task = store
        .snapshot
        .tasks
        .get(&input.task_id)
        .cloned()
        .ok_or(EscrowLifecycleError::NotFound)?;
    validate_funding(actor, &input, &task)?;
    let escrow_id = next_escrow_id(store, payload);
    let record = build_record(escrow_id.as_str(), actor, input);
    store.snapshot.escrows.insert(escrow_id.clone(), record);
    store.persist().map_err(persistence)?;
    Ok(escrow_status_response(&store.snapshot.escrows[&escrow_id]))
}

pub(super) fn authorize_release(
    store: &mut ServiceApiMessageStore,
    actor: &str,
    escrow_id: &str,
) -> Result<ServiceApiEscrowStatusBody, EscrowLifecycleError> {
    store.refresh_from_disk().map_err(persistence)?;
    let escrow = store
        .snapshot
        .escrows
        .get(escrow_id)
        .cloned()
        .ok_or(EscrowLifecycleError::NotFound)?;
    validate_release(store, actor, &escrow)?;
    let record = store.snapshot.escrows.get_mut(escrow_id).unwrap();
    record.state = "release-authorized".to_owned();
    let response = escrow_status_response(record);
    store.persist().map_err(persistence)?;
    Ok(response)
}

fn parse_fund(payload: &str) -> Result<FundInput, EscrowLifecycleError> {
    let input: FundInput = serde_json::from_str(payload)
        .map_err(|error| bad("ESCROW_AGREEMENT_INVALID", error.to_string()))?;
    if input.amount_lamports == 0 || input.amount_lamports > MAX_AMOUNT_LAMPORTS {
        return Err(bad("ESCROW_AMOUNT_INVALID", "amount is outside MVP bounds"));
    }
    if input.network != "solana-devnet" {
        return Err(bad(
            "ESCROW_NETWORK_INVALID",
            "network must be solana-devnet",
        ));
    }
    if input.idempotency_key.trim().is_empty() || input.release_policy != "task-completed" {
        return Err(bad(
            "ESCROW_AGREEMENT_INVALID",
            "escrow policy or retry key is invalid",
        ));
    }
    Ok(input)
}

fn validate_funding(
    actor: &str,
    input: &FundInput,
    task: &ServiceApiPersistedTaskRecord,
) -> Result<(), EscrowLifecycleError> {
    if task.state != "accepted" {
        return Err(conflict(
            "ESCROW_TASK_STATE_CONFLICT",
            "task is not accepted",
        ));
    }
    if task.creator_did.as_deref() != Some(actor) {
        return Err(forbidden(
            "ESCROW_FUNDER_MISMATCH",
            "funder is not task creator",
        ));
    }
    if input.release_authority_did != actor {
        return Err(forbidden(
            "ESCROW_RELEASE_AUTHORITY_MISMATCH",
            "release authority differs",
        ));
    }
    if task.provider_did.as_deref() != Some(input.beneficiary_did.as_str()) {
        return Err(conflict(
            "ESCROW_BENEFICIARY_MISMATCH",
            "beneficiary differs",
        ));
    }
    if task.transaction_id.as_deref() != Some(input.transaction_id.as_str()) {
        return Err(conflict(
            "ESCROW_TRANSACTION_MISMATCH",
            "transaction differs",
        ));
    }
    if task.terms_digest.as_deref() != Some(input.terms_digest.as_str()) {
        return Err(conflict("ESCROW_TERMS_MISMATCH", "terms differ"));
    }
    Ok(())
}

fn validate_release(
    store: &ServiceApiMessageStore,
    actor: &str,
    escrow: &ServiceApiPersistedEscrowRecord,
) -> Result<(), EscrowLifecycleError> {
    if escrow.release_authority_did.as_deref() != Some(actor) {
        return Err(forbidden(
            "ESCROW_RELEASE_AUTHORITY_MISMATCH",
            "actor is not release authority",
        ));
    }
    let task_id = escrow.task_id.as_deref().ok_or_else(migration_required)?;
    let task = store
        .snapshot
        .tasks
        .get(task_id)
        .ok_or(EscrowLifecycleError::NotFound)?;
    if task.state != "completed" || task.completion_evidence_digest.is_none() {
        return Err(conflict(
            "ESCROW_RELEASE_NOT_ELIGIBLE",
            "task is not completed with evidence",
        ));
    }
    Ok(())
}

fn build_record(id: &str, actor: &str, input: FundInput) -> ServiceApiPersistedEscrowRecord {
    ServiceApiPersistedEscrowRecord {
        escrow_id: id.to_owned(),
        state: "funded".to_owned(),
        task_id: Some(input.task_id),
        transaction_id: Some(input.transaction_id),
        funder_did: Some(actor.to_owned()),
        beneficiary_did: Some(input.beneficiary_did),
        amount_lamports: Some(input.amount_lamports),
        network: Some(input.network),
        terms_digest: Some(input.terms_digest),
        release_authority_did: Some(input.release_authority_did),
        release_policy: Some(input.release_policy),
        fund_idempotency_key: Some(input.idempotency_key),
        settlement: ServiceApiSettlementMetadata::default(),
    }
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
