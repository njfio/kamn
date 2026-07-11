use super::super::super::*;
use super::{next_task_id, persist_task_created_audit_export};

const DIGEST_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TaskLifecycleError {
    BadRequest(&'static str, String),
    Forbidden(&'static str, String),
    Conflict(&'static str, String),
    NotFound,
    Persistence(String),
}

#[derive(Debug, Deserialize)]
struct CreateInput {
    provider_did: String,
    transaction_id: String,
    terms_digest: String,
    idempotency_key: String,
    #[serde(default)]
    creator: Option<String>,
    #[serde(default)]
    task_type: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TransitionInput {
    idempotency_key: String,
    #[serde(default)]
    completion_evidence_digest: Option<String>,
}

pub(super) fn create_bound_task(
    store: &mut ServiceApiMessageStore,
    actor_did: &str,
    payload: &str,
) -> Result<ServiceApiTaskCreateBody, TaskLifecycleError> {
    store.refresh_from_disk().map_err(persistence)?;
    let input = parse_create_input(payload, actor_did)?;
    require_registered_provider(store, input.provider_did.as_str())?;
    if let Some(existing) = find_create_retry(store, actor_did, &input)? {
        return Ok(create_response(existing));
    }
    let task_id = next_task_id(store, payload);
    let record = bound_task_record(task_id.as_str(), actor_did, &input);
    store.snapshot.tasks.insert(task_id.clone(), record);
    issue_lifecycle_grants(
        store,
        task_id.as_str(),
        actor_did,
        input.provider_did.as_str(),
    );
    store.persist().map_err(persistence)?;
    persist_task_created_audit_export(store, task_id.as_str()).map_err(persistence)?;
    Ok(create_response(&store.snapshot.tasks[&task_id]))
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
    let input = parse_transition_input(payload, action)?;
    if let Some(receipt) = find_transition_retry(store, actor_did, task_id, action, &input)? {
        return Ok(transition_response_from_receipt(store, receipt));
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
    let receipt = build_transition_receipt(
        record,
        actor_did,
        action,
        prior_state,
        input.idempotency_key,
        correlation_id,
        receipt_sequence,
    )?;
    let response = transition_response(record, receipt.receipt_id.as_str());
    store.snapshot.task_transition_receipts.push(receipt);
    store.persist().map_err(persistence)?;
    Ok(response)
}

fn find_create_retry<'a>(
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

fn same_agreement(record: &ServiceApiPersistedTaskRecord, input: &CreateInput) -> bool {
    record.provider_did.as_deref() == Some(input.provider_did.as_str())
        && record.transaction_id.as_deref() == Some(input.transaction_id.as_str())
        && record.terms_digest.as_deref() == Some(input.terms_digest.as_str())
        && record.task_type == input.task_type
        && record.description == input.description
}

fn find_transition_retry<'a>(
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
    if receipt.task_id == task_id
        && receipt.action == action
        && receipt.completion_evidence_digest == input.completion_evidence_digest
    {
        return Ok(Some(receipt));
    }
    Err(conflict(
        "TASK_IDEMPOTENCY_CONFLICT",
        "transition idempotency key was reused",
    ))
}

fn build_transition_receipt(
    record: &ServiceApiPersistedTaskRecord,
    actor: &str,
    action: &str,
    prior_state: String,
    idempotency_key: String,
    correlation_id: &str,
    sequence: usize,
) -> Result<ServiceApiTaskTransitionReceiptRecord, TaskLifecycleError> {
    Ok(ServiceApiTaskTransitionReceiptRecord {
        receipt_id: format!("task-transition-receipt-{sequence:08}"),
        correlation_id: correlation_id.to_owned(),
        idempotency_key,
        actor_did: actor.to_owned(),
        task_id: record.task_id.clone(),
        transaction_id: required_agreement(&record.transaction_id)?,
        action: action.to_owned(),
        prior_state,
        resulting_state: record.state.clone(),
        terms_digest: required_agreement(&record.terms_digest)?,
        completion_evidence_digest: record.completion_evidence_digest.clone(),
    })
}

fn required_agreement(value: &Option<String>) -> Result<String, TaskLifecycleError> {
    value.clone().ok_or_else(|| {
        conflict(
            "TASK_AGREEMENT_MIGRATION_REQUIRED",
            "legacy task agreement is incomplete",
        )
    })
}

fn create_response(record: &ServiceApiPersistedTaskRecord) -> ServiceApiTaskCreateBody {
    ServiceApiTaskCreateBody {
        task_id: record.task_id.clone(),
        state: record.state.clone(),
        transaction_id: record.transaction_id.clone(),
        creator_did: record.creator_did.clone(),
        provider_did: record.provider_did.clone(),
        terms_digest: record.terms_digest.clone(),
    }
}

fn transition_response(
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

fn transition_response_from_receipt(
    store: &ServiceApiMessageStore,
    receipt: &ServiceApiTaskTransitionReceiptRecord,
) -> ServiceApiTaskTransitionBody {
    let record = &store.snapshot.tasks[&receipt.task_id];
    transition_response(record, receipt.receipt_id.as_str())
}

fn parse_create_input(payload: &str, actor: &str) -> Result<CreateInput, TaskLifecycleError> {
    let input: CreateInput =
        serde_json::from_str(payload).map_err(|error| agreement(error.to_string()))?;
    AgentDid::parse(input.provider_did.as_str())
        .map_err(|error| bad("TASK_PROVIDER_INVALID", error.to_string()))?;
    if input
        .creator
        .as_deref()
        .is_some_and(|creator| creator != actor)
    {
        return Err(bad(
            "TASK_CREATOR_MISMATCH",
            "body creator differs from authenticated actor",
        ));
    }
    if input.transaction_id.trim().is_empty()
        || input.idempotency_key.trim().is_empty()
        || !valid_digest(&input.terms_digest)
    {
        return Err(agreement(
            "transaction, terms digest, and idempotency key are required",
        ));
    }
    Ok(input)
}

fn parse_transition_input(
    payload: &str,
    action: &str,
) -> Result<TransitionInput, TaskLifecycleError> {
    let input: TransitionInput = serde_json::from_str(payload)
        .map_err(|error| bad("TASK_AGREEMENT_INVALID", error.to_string()))?;
    if input.idempotency_key.trim().is_empty() {
        return Err(bad(
            "TASK_AGREEMENT_INVALID",
            "transition idempotency key is required",
        ));
    }
    if action == "task:complete"
        && !input
            .completion_evidence_digest
            .as_deref()
            .is_some_and(valid_digest)
    {
        return Err(bad(
            "TASK_COMPLETION_EVIDENCE_INVALID",
            "completion evidence digest is required",
        ));
    }
    Ok(input)
}

fn require_registered_provider(
    store: &ServiceApiMessageStore,
    did: &str,
) -> Result<(), TaskLifecycleError> {
    if store
        .snapshot
        .agents
        .get(did)
        .is_some_and(|agent| agent.registered)
    {
        return Ok(());
    }
    Err(bad(
        "TASK_PROVIDER_NOT_REGISTERED",
        "task provider is not registered",
    ))
}

fn require_bound_provider(
    record: &ServiceApiPersistedTaskRecord,
    actor: &str,
) -> Result<(), TaskLifecycleError> {
    let provider = record.provider_did.as_deref().ok_or_else(|| {
        conflict(
            "TASK_AGREEMENT_MIGRATION_REQUIRED",
            "legacy task agreement is incomplete",
        )
    })?;
    if provider == actor {
        return Ok(());
    }
    Err(TaskLifecycleError::Forbidden(
        "TASK_PROVIDER_MISMATCH",
        "actor is not the assigned provider".to_owned(),
    ))
}

fn require_legal_transition(state: &str, action: &str) -> Result<&'static str, TaskLifecycleError> {
    match (state, action) {
        ("submitted", "task:accept") => Ok("accepted"),
        ("accepted", "task:complete") => Ok("completed"),
        _ => Err(conflict(
            "TASK_STATE_CONFLICT",
            "task transition is not legal from current state",
        )),
    }
}

fn bound_task_record(
    id: &str,
    creator: &str,
    input: &CreateInput,
) -> ServiceApiPersistedTaskRecord {
    ServiceApiPersistedTaskRecord {
        task_id: id.to_owned(),
        state: "submitted".to_owned(),
        creator_did: Some(creator.to_owned()),
        task_type: input.task_type.clone(),
        description: input.description.clone(),
        assignee: Some(input.provider_did.clone()),
        provider_did: Some(input.provider_did.clone()),
        transaction_id: Some(input.transaction_id.clone()),
        terms_digest: Some(input.terms_digest.clone()),
        completion_evidence_digest: None,
        create_idempotency_key: Some(input.idempotency_key.clone()),
    }
}

fn issue_lifecycle_grants(
    store: &mut ServiceApiMessageStore,
    task_id: &str,
    creator: &str,
    provider: &str,
) {
    for (did, action, role) in [
        (creator, "task:read", "participant"),
        (provider, "task:read", "participant"),
        (provider, "task:accept", "provider"),
        (provider, "task:complete", "provider"),
    ] {
        let key = format!("task-lifecycle:{task_id}:{did}:{action}");
        store.snapshot.agent_grants.insert(
            key.clone(),
            ServiceApiPersistedAgentGrantRecord {
                did: did.to_owned(),
                resource: format!("task:{task_id}"),
                role: role.to_owned(),
                action: action.to_owned(),
                status: "active".to_owned(),
                idempotency_key: key,
            },
        );
    }
}

fn valid_digest(value: &str) -> bool {
    value.len() == DIGEST_LEN
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
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
