use super::*;

pub(super) fn mutation_entries(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    task: &ServiceApiPersistedTaskRecord,
    escrow: &ServiceApiPersistedEscrowRecord,
    tasks: &[&ServiceApiTaskTransitionReceiptRecord],
    escrows: &[&ServiceApiEscrowTransitionReceiptRecord],
) -> Result<Vec<ReceiptChainEntry>, TaskProjectionError> {
    let mut entries = vec![task_entry(snapshot, task, tasks[0])?];
    entries.push(task_entry(snapshot, task, tasks[1])?);
    entries.push(escrow_entry(snapshot, task, escrow, escrows[0])?);
    if let Some(receipt) = tasks.get(2) {
        entries.push(task_entry(snapshot, task, receipt)?);
    }
    if let Some(receipt) = escrows.get(1) {
        entries.push(escrow_entry(snapshot, task, escrow, receipt)?);
    }
    Ok(entries)
}

fn task_entry(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    task: &ServiceApiPersistedTaskRecord,
    receipt: &ServiceApiTaskTransitionReceiptRecord,
) -> Result<ReceiptChainEntry, TaskProjectionError> {
    let actor = binding::task_actor(task, receipt.action.as_str())?;
    binding::require_task(task, receipt, actor)?;
    build_entry(snapshot, MutationReceipt::Task(receipt), actor)
}

fn escrow_entry(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    task: &ServiceApiPersistedTaskRecord,
    escrow: &ServiceApiPersistedEscrowRecord,
    receipt: &ServiceApiEscrowTransitionReceiptRecord,
) -> Result<ReceiptChainEntry, TaskProjectionError> {
    let actor = binding::escrow_actor(escrow, receipt.action.as_str())?;
    binding::require_escrow(task, escrow, receipt, actor)?;
    build_entry(snapshot, MutationReceipt::Escrow(receipt), actor)
}

enum MutationReceipt<'a> {
    Task(&'a ServiceApiTaskTransitionReceiptRecord),
    Escrow(&'a ServiceApiEscrowTransitionReceiptRecord),
}

impl MutationReceipt<'_> {
    fn receipt_id(&self) -> &str {
        match self {
            Self::Task(receipt) => &receipt.receipt_id,
            Self::Escrow(receipt) => &receipt.receipt_id,
        }
    }

    fn action(&self) -> &str {
        match self {
            Self::Task(receipt) => &receipt.action,
            Self::Escrow(receipt) => &receipt.action,
        }
    }

    fn authorization_action(&self) -> &str {
        match self.action() {
            "escrow:release-authorize" => "escrow:release",
            action => action,
        }
    }

    fn resource_id(&self) -> &str {
        match self {
            Self::Task(receipt) => &receipt.task_id,
            Self::Escrow(receipt) => &receipt.escrow_id,
        }
    }

    fn authorization_resource(&self) -> String {
        match self {
            Self::Task(receipt) if receipt.action == "task:create" => "transaction:new".to_owned(),
            Self::Task(receipt) => format!("task:{}", receipt.task_id),
            Self::Escrow(receipt) if receipt.action == "escrow:fund" => {
                format!("task:{}", receipt.task_id)
            }
            Self::Escrow(receipt) => format!("escrow:{}", receipt.escrow_id),
        }
    }

    fn correlation_id(&self) -> &str {
        match self {
            Self::Task(receipt) => &receipt.correlation_id,
            Self::Escrow(receipt) => &receipt.correlation_id,
        }
    }

    fn idempotency_key(&self) -> &str {
        match self {
            Self::Task(receipt) => &receipt.idempotency_key,
            Self::Escrow(receipt) => &receipt.idempotency_key,
        }
    }

    fn states(&self) -> (&str, &str) {
        match self {
            Self::Task(receipt) => (&receipt.prior_state, &receipt.resulting_state),
            Self::Escrow(receipt) => (&receipt.prior_state, &receipt.resulting_state),
        }
    }

    fn digest(&self) -> String {
        match self {
            Self::Task(receipt) => authority_digest::task(receipt),
            Self::Escrow(receipt) => authority_digest::escrow(receipt),
        }
    }
}

fn build_entry(
    snapshot: &ServiceApiPersistedMessageStoreSnapshot,
    receipt: MutationReceipt<'_>,
    actor: &str,
) -> Result<ReceiptChainEntry, TaskProjectionError> {
    let authorization = authorized_receipt(snapshot, &receipt, actor)?;
    let (prior_state, resulting_state) = receipt.states();
    Ok(ReceiptChainEntry {
        receipt_id: receipt.receipt_id().to_owned(),
        receipt_digest: receipt.digest(),
        authorization_digest: authority_digest::authorization(authorization),
        actor_did: actor.to_owned(),
        action: receipt.action().to_owned(),
        resource_id: receipt.resource_id().to_owned(),
        correlation_id: receipt.correlation_id().to_owned(),
        idempotency_key: receipt.idempotency_key().to_owned(),
        prior_state: prior_state.to_owned(),
        resulting_state: resulting_state.to_owned(),
    })
}

fn authorized_receipt<'a>(
    snapshot: &'a ServiceApiPersistedMessageStoreSnapshot,
    receipt: &MutationReceipt<'_>,
    actor: &str,
) -> Result<&'a ServiceApiAuthorizationReceiptRecord, TaskProjectionError> {
    let resource = receipt.authorization_resource();
    binding::authorization(
        snapshot,
        actor,
        receipt.authorization_action(),
        resource.as_str(),
    )
}
