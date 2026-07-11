use super::*;
use crate::service_api_endpoint::projection_models::{
    ServiceApiTaskPublicProjection, TASK_PROJECTION_SCHEMA_VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TaskProjectionError {
    Unregistered,
    Forbidden,
    EscrowBindingMissing,
    Inconsistent,
    Persistence(String),
}

pub(crate) fn participant(
    store: &mut ServiceApiMessageStore,
    task_id: &str,
    requester_did: &str,
) -> Result<Option<ServiceApiParticipantTaskProjection>, TaskProjectionError> {
    store
        .refresh_from_disk()
        .map_err(TaskProjectionError::Persistence)?;
    require_registered(&store.snapshot, requester_did)?;
    let Some(task) = store.snapshot.tasks.get(task_id) else {
        return Ok(None);
    };
    let role = participant_role(task, requester_did).ok_or(TaskProjectionError::Forbidden)?;
    let public = build_public_projection(&store.snapshot, task)?;
    let receipts = task_receipt_ids(&store.snapshot, task_id);
    Ok(Some(ServiceApiParticipantTaskProjection {
        view_scope: "participant-private".to_owned(),
        participant_role: role.to_owned(),
        public,
        task_receipt_ids: receipts,
        completion_evidence_digest: task.completion_evidence_digest.clone(),
    }))
}

pub(crate) fn verifier(
    store: &mut ServiceApiMessageStore,
    task_id: &str,
    requester_did: &str,
) -> Result<Option<ServiceApiVerifierTaskProjection>, TaskProjectionError> {
    store
        .refresh_from_disk()
        .map_err(TaskProjectionError::Persistence)?;
    require_registered(&store.snapshot, requester_did)?;
    let Some(task) = store.snapshot.tasks.get(task_id) else {
        return Ok(None);
    };
    Ok(Some(ServiceApiVerifierTaskProjection {
        view_scope: "restricted-public".to_owned(),
        public: build_public_projection(&store.snapshot, task)?,
    }))
}

fn require_registered(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    requester_did: &str,
) -> Result<(), TaskProjectionError> {
    if snapshot
        .agents
        .get(requester_did)
        .is_some_and(|agent| agent.registered)
    {
        return Ok(());
    }
    Err(TaskProjectionError::Unregistered)
}

fn participant_role<'a>(
    task: &'a ServiceApiPersistedTaskRecord,
    requester: &str,
) -> Option<&'a str> {
    if task.creator_did.as_deref() == Some(requester) {
        return Some("creator");
    }
    (task.provider_did.as_deref() == Some(requester)).then_some("provider")
}

fn build_public_projection(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    task: &ServiceApiPersistedTaskRecord,
) -> Result<ServiceApiTaskPublicProjection, TaskProjectionError> {
    let escrow = bound_escrow(snapshot, task)?;
    let transaction_id = matching_transaction_id(task, escrow)?;
    let mut projection = public_fields(task, escrow, transaction_id)?;
    projection.public_commitment = public_commitment(&projection);
    Ok(projection)
}

fn bound_escrow<'a>(
    snapshot: &'a ServiceApiPersistedMessageStoreSnapshot,
    task: &ServiceApiPersistedTaskRecord,
) -> Result<&'a ServiceApiPersistedEscrowRecord, TaskProjectionError> {
    let mut matches = snapshot
        .escrows
        .values()
        .filter(|escrow| escrow.task_id.as_deref() == Some(task.task_id.as_str()));
    let escrow = matches
        .next()
        .ok_or(TaskProjectionError::EscrowBindingMissing)?;
    if matches.next().is_some() {
        return Err(TaskProjectionError::Inconsistent);
    }
    Ok(escrow)
}

fn matching_transaction_id<'a>(
    task: &'a ServiceApiPersistedTaskRecord,
    escrow: &'a ServiceApiPersistedEscrowRecord,
) -> Result<&'a str, TaskProjectionError> {
    match (
        task.transaction_id.as_deref(),
        escrow.transaction_id.as_deref(),
    ) {
        (Some(task_id), Some(escrow_id)) if task_id == escrow_id => Ok(task_id),
        _ => Err(TaskProjectionError::Inconsistent),
    }
}

fn public_fields(
    task: &ServiceApiPersistedTaskRecord,
    escrow: &ServiceApiPersistedEscrowRecord,
    transaction_id: &str,
) -> Result<ServiceApiTaskPublicProjection, TaskProjectionError> {
    let amount_lamports = escrow
        .amount_lamports
        .ok_or(TaskProjectionError::Inconsistent)?;
    let network = escrow
        .network
        .clone()
        .ok_or(TaskProjectionError::Inconsistent)?;
    Ok(ServiceApiTaskPublicProjection {
        schema_version: TASK_PROJECTION_SCHEMA_VERSION.to_owned(),
        task_id: task.task_id.clone(),
        transaction_id: transaction_id.to_owned(),
        task_state: task.state.clone(),
        escrow_id: escrow.escrow_id.clone(),
        escrow_state: escrow.state.clone(),
        amount_lamports,
        network,
        settlement_tx_signature: escrow.settlement.settlement_tx_signature.clone(),
        settlement_commitment: escrow.settlement.settlement_commitment.clone(),
        public_commitment: String::new(),
    })
}

fn public_commitment(projection: &ServiceApiTaskPublicProjection) -> String {
    let canonical = format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}",
        projection.task_id,
        projection.transaction_id,
        projection.task_state,
        projection.escrow_id,
        projection.escrow_state,
        projection.amount_lamports,
        projection.network,
        projection.settlement_tx_signature.as_deref().unwrap_or(""),
        projection.settlement_commitment.as_deref().unwrap_or("")
    );
    format!(
        "fnv1a64:{:016x}",
        crate::service_api_endpoint::deterministic_body_tag(canonical.as_bytes())
    )
}

fn task_receipt_ids(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    task_id: &str,
) -> Vec<String> {
    snapshot
        .task_transition_receipts
        .iter()
        .filter(|receipt| receipt.task_id == task_id)
        .map(|receipt| receipt.receipt_id.clone())
        .collect()
}
