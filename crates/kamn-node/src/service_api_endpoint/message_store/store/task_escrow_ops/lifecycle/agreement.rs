use super::input::CreateInput;
use super::*;

pub(super) fn require_registered_provider(
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

pub(super) fn require_bound_provider(
    record: &ServiceApiPersistedTaskRecord,
    actor: &str,
) -> Result<(), TaskLifecycleError> {
    let provider = required_agreement(&record.provider_did)?;
    if provider == actor {
        return Ok(());
    }
    Err(TaskLifecycleError::Forbidden(
        "TASK_PROVIDER_MISMATCH",
        "actor is not the assigned provider".to_owned(),
    ))
}

pub(super) fn require_legal_transition(
    state: &str,
    action: &str,
) -> Result<&'static str, TaskLifecycleError> {
    match (state, action) {
        ("submitted", "task:accept") => Ok("accepted"),
        ("accepted", "task:complete") => Ok("completed"),
        _ => Err(conflict(
            "TASK_STATE_CONFLICT",
            "task transition is not legal from current state",
        )),
    }
}

pub(super) fn build_record(
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

pub(super) fn issue_grants(
    store: &mut ServiceApiMessageStore,
    task_id: &str,
    creator: &str,
    provider: &str,
) {
    for (did, action, role) in grant_contract(creator, provider) {
        let resource = format!("task:{task_id}");
        let key = format!("task-lifecycle:{task_id}:{did}:{action}");
        store.snapshot.agent_grants.insert(
            key.clone(),
            ServiceApiPersistedAgentGrantRecord {
                did: did.to_owned(),
                resource,
                role: role.to_owned(),
                action: action.to_owned(),
                status: "active".to_owned(),
                idempotency_key: key,
            },
        );
    }
}

fn grant_contract<'a>(
    creator: &'a str,
    provider: &'a str,
) -> [(&'a str, &'static str, &'static str); 5] {
    [
        (creator, "task:read", "participant"),
        (provider, "task:read", "participant"),
        (provider, "task:accept", "provider"),
        (provider, "task:complete", "provider"),
        (creator, "escrow:fund", "initiator"),
    ]
}

pub(super) fn required_agreement(value: &Option<String>) -> Result<String, TaskLifecycleError> {
    value.clone().ok_or_else(|| {
        conflict(
            "TASK_AGREEMENT_MIGRATION_REQUIRED",
            "legacy task agreement is incomplete",
        )
    })
}
