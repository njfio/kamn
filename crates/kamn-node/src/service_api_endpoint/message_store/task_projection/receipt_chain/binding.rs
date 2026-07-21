use super::*;

pub(super) fn authorization<'a>(
    snapshot: &'a ServiceApiPersistedMessageStoreSnapshot,
    actor: &str,
    action: &str,
    resource: &str,
) -> Result<&'a ServiceApiAuthorizationReceiptRecord, TaskProjectionError> {
    snapshot
        .authorization_receipts
        .iter()
        .find(|receipt| allowed(receipt, actor, action, resource))
        .ok_or(TaskProjectionError::ReceiptChainInvalid)
}

fn allowed(
    receipt: &ServiceApiAuthorizationReceiptRecord,
    actor: &str,
    action: &str,
    resource: &str,
) -> bool {
    receipt.actor_did == actor
        && receipt.action == action
        && receipt.resource == resource
        && receipt.decision == "allow"
}

pub(super) fn task_actor<'a>(
    task: &'a ServiceApiPersistedTaskRecord,
    action: &str,
) -> Result<&'a str, TaskProjectionError> {
    match action {
        "task:create" => required(&task.creator_did),
        "task:accept" | "task:complete" => required(&task.provider_did),
        _ => invalid(),
    }
}

pub(super) fn escrow_actor<'a>(
    escrow: &'a ServiceApiPersistedEscrowRecord,
    action: &str,
) -> Result<&'a str, TaskProjectionError> {
    match action {
        "escrow:fund" => required(&escrow.funder_did),
        "escrow:release-authorize" => required(&escrow.release_authority_did),
        _ => invalid(),
    }
}

pub(super) fn require_task(
    task: &ServiceApiPersistedTaskRecord,
    receipt: &ServiceApiTaskTransitionReceiptRecord,
    actor: &str,
) -> Result<(), TaskProjectionError> {
    let valid = receipt.actor_did == actor
        && task.transaction_id.as_deref() == Some(receipt.transaction_id.as_str())
        && task.terms_digest.as_deref() == Some(receipt.terms_digest.as_str())
        && completion_evidence_matches(task, receipt);
    valid
        .then_some(())
        .ok_or(TaskProjectionError::ReceiptChainInvalid)
}

fn completion_evidence_matches(
    task: &ServiceApiPersistedTaskRecord,
    receipt: &ServiceApiTaskTransitionReceiptRecord,
) -> bool {
    if receipt.action == "task:complete" {
        return receipt.completion_evidence_digest == task.completion_evidence_digest;
    }
    receipt.completion_evidence_digest.is_none()
}

pub(super) fn require_escrow(
    task: &ServiceApiPersistedTaskRecord,
    escrow: &ServiceApiPersistedEscrowRecord,
    receipt: &ServiceApiEscrowTransitionReceiptRecord,
    actor: &str,
) -> Result<(), TaskProjectionError> {
    let valid = receipt.actor_did == actor
        && receipt.task_id == task.task_id
        && escrow.transaction_id.as_deref() == Some(receipt.transaction_id.as_str())
        && escrow.terms_digest.as_deref() == Some(receipt.terms_digest.as_str())
        && escrow.network.as_deref() == Some(receipt.network.as_str())
        && escrow.amount_lamports == Some(receipt.amount_lamports)
        && escrow.release_policy.as_deref() == Some(receipt.release_policy.as_str());
    valid
        .then_some(())
        .ok_or(TaskProjectionError::ReceiptChainInvalid)
}

fn required(value: &Option<String>) -> Result<&str, TaskProjectionError> {
    value
        .as_deref()
        .ok_or(TaskProjectionError::ReceiptChainInvalid)
}

fn invalid<T>() -> Result<T, TaskProjectionError> {
    Err(TaskProjectionError::ReceiptChainInvalid)
}
