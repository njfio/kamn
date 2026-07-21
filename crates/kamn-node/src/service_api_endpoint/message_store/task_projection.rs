use super::*;
use crate::service_api_endpoint::projection_models::{
    ServiceApiTaskPublicProjection, TASK_PROJECTION_SCHEMA_VERSION,
};

mod commitment;
mod receipt_chain;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TaskProjectionError {
    Unregistered,
    Forbidden,
    EscrowBindingMissing,
    Inconsistent,
    ReceiptChainInvalid,
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
    let (public, chain) = build_projection(&store.snapshot, task)?;
    Ok(Some(ServiceApiParticipantTaskProjection {
        view_scope: "participant-private".to_owned(),
        participant_role: role.to_owned(),
        public,
        task_receipt_ids: chain.task_receipt_ids(requester_did),
        receipt_chain_receipts: chain.participant_receipts(requester_did),
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
    let (public, _) = build_projection(&store.snapshot, task)?;
    Ok(Some(ServiceApiVerifierTaskProjection {
        view_scope: "restricted-public".to_owned(),
        public,
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

fn build_projection(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    task: &ServiceApiPersistedTaskRecord,
) -> Result<(ServiceApiTaskPublicProjection, receipt_chain::ReceiptChain), TaskProjectionError> {
    let escrow = bound_escrow(snapshot, task)?;
    let transaction_id = matching_transaction_id(task, escrow)?;
    require_settlement_consistency(snapshot, escrow)?;
    let chain = receipt_chain::derive(snapshot, task, escrow)?;
    let mut projection = public_fields(task, escrow, transaction_id)?;
    projection.receipt_chain_commitment = chain.commitment.clone();
    projection.public_commitment = commitment::public_commitment(&projection);
    Ok((projection, chain))
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

fn require_settlement_consistency(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    escrow: &ServiceApiPersistedEscrowRecord,
) -> Result<(), TaskProjectionError> {
    let Some(signature) = escrow.settlement.settlement_tx_signature.as_deref() else {
        return Ok(());
    };
    let intent = snapshot
        .settlement_intents
        .get(&escrow.escrow_id)
        .ok_or(TaskProjectionError::Inconsistent)?;
    if intent.state == "confirmed"
        && intent.expected_signature == signature
        && Some(intent.amount_lamports) == escrow.amount_lamports
        && intent.network == "solana:devnet"
        && escrow.network.as_deref() == Some("solana-devnet")
    {
        return Ok(());
    }
    Err(TaskProjectionError::Inconsistent)
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
        receipt_chain_commitment: String::new(),
        public_commitment: String::new(),
    })
}
