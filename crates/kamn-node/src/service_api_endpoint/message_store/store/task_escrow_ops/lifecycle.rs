use super::super::super::*;
use super::{next_task_id, persist_task_created_audit_export};

mod agreement;
mod idempotency;
mod input;
mod receipt;

use agreement::require_registered_provider;
use agreement::{build_record, issue_grants, require_bound_provider, require_legal_transition};
use idempotency::{find_create_retry, find_transition_retry};
use input::{parse_create, parse_transition};
use receipt::{create_response, retry_response, transition_response, ReceiptInput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TaskLifecycleError {
    BadRequest(&'static str, String),
    Forbidden(&'static str, String),
    Conflict(&'static str, String),
    NotFound,
    Persistence(String),
}

pub(super) fn create_bound_task(
    store: &mut ServiceApiMessageStore,
    actor_did: &str,
    payload: &str,
    correlation_id: &str,
) -> Result<ServiceApiTaskCreateBody, TaskLifecycleError> {
    store.refresh_from_disk().map_err(persistence)?;
    let input = parse_create(payload, actor_did)?;
    require_registered_provider(store, input.provider_did.as_str())?;
    if let Some(existing) = find_create_retry(store, actor_did, &input)? {
        let record = existing.clone();
        let receipt = creation_receipt(store, record.task_id.as_str())?;
        return Ok(create_response(&record, receipt));
    }
    let task_id = next_task_id(store, payload);
    let record = build_record(task_id.as_str(), actor_did, &input);
    let receipt = receipt::build(
        &record,
        ReceiptInput {
            actor: actor_did,
            action: "task:create",
            prior_state: "none".to_owned(),
            idempotency_key: input.idempotency_key.clone(),
            correlation_id,
            sequence: store.snapshot.task_transition_receipts.len() + 1,
        },
    )?;
    store.snapshot.tasks.insert(task_id.clone(), record);
    store.snapshot.task_transition_receipts.push(receipt);
    issue_grants(
        store,
        task_id.as_str(),
        actor_did,
        input.provider_did.as_str(),
    );
    store.persist().map_err(persistence)?;
    persist_task_created_audit_export(store, task_id.as_str()).map_err(persistence)?;
    let record = &store.snapshot.tasks[&task_id];
    let receipt = creation_receipt(store, task_id.as_str())?;
    Ok(create_response(record, receipt))
}

pub(super) fn transition_bound_task(
    store: &mut ServiceApiMessageStore,
    actor_did: &str,
    task_id: &str,
    action: &str,
    payload: &str,
    correlation_id: &str,
) -> Result<ServiceApiTaskTransitionBody, TaskLifecycleError> {
    store.refresh_from_disk().map_err(persistence)?;
    if !store.snapshot.tasks.contains_key(task_id) {
        return Err(TaskLifecycleError::NotFound);
    }
    let input = parse_transition(payload, action)?;
    if let Some(receipt) = find_transition_retry(store, actor_did, task_id, action, &input)? {
        return Ok(retry_response(store, receipt));
    }
    let receipt_sequence = store.snapshot.task_transition_receipts.len() + 1;
    let record = store
        .snapshot
        .tasks
        .get_mut(task_id)
        .ok_or(TaskLifecycleError::NotFound)?;
    require_bound_provider(record, actor_did)?;
    let target = require_legal_transition(record.state.as_str(), action)?;
    let prior_state = record.state.clone();
    record.state = target.to_owned();
    if action == "task:complete" {
        record.completion_evidence_digest = input.completion_evidence_digest;
    }
    let receipt = receipt::build(
        record,
        ReceiptInput {
            actor: actor_did,
            action,
            prior_state,
            idempotency_key: input.idempotency_key,
            correlation_id,
            sequence: receipt_sequence,
        },
    )?;
    let response = transition_response(record, &receipt);
    store.snapshot.task_transition_receipts.push(receipt);
    store.persist().map_err(persistence)?;
    Ok(response)
}

fn creation_receipt<'a>(
    store: &'a ServiceApiMessageStore,
    task_id: &str,
) -> Result<&'a ServiceApiTaskTransitionReceiptRecord, TaskLifecycleError> {
    store
        .snapshot
        .task_transition_receipts
        .iter()
        .find(|receipt| receipt.task_id == task_id && receipt.action == "task:create")
        .ok_or_else(|| conflict("TASK_RECEIPT_MISSING", "task creation receipt is missing"))
}

fn bad(code: &'static str, message: impl Into<String>) -> TaskLifecycleError {
    TaskLifecycleError::BadRequest(code, message.into())
}
fn agreement(message: impl Into<String>) -> TaskLifecycleError {
    bad("TASK_AGREEMENT_INVALID", message)
}
fn conflict(code: &'static str, message: impl Into<String>) -> TaskLifecycleError {
    TaskLifecycleError::Conflict(code, message.into())
}
fn persistence(error: String) -> TaskLifecycleError {
    TaskLifecycleError::Persistence(error)
}
