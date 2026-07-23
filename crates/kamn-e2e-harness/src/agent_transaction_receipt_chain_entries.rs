use std::collections::BTreeSet;

use super::agent_transaction_receipt_chain::invalid;
use super::agent_transaction_receipt_chain_digest as digest;
use super::agent_transaction_receipt_chain_fields::Mutation;
use super::agent_transaction_receipt_chain_model::*;

pub(super) fn mutation_entries(
    state: &State,
    task: &Task,
    escrow: &Escrow,
    tasks: &[&TaskReceipt],
    escrows: &[&EscrowReceipt],
) -> Result<Vec<ChainEntry>, String> {
    Ok(vec![
        task_entry(state, task, tasks[0])?,
        task_entry(state, task, tasks[1])?,
        escrow_entry(state, task, escrow, escrows[0])?,
        task_entry(state, task, tasks[2])?,
        escrow_entry(state, task, escrow, escrows[1])?,
    ])
}

fn task_entry(state: &State, task: &Task, receipt: &TaskReceipt) -> Result<ChainEntry, String> {
    let actor = match receipt.action.as_str() {
        "task:create" => task.creator_did.as_deref(),
        "task:accept" | "task:complete" => task.provider_did.as_deref(),
        _ => None,
    }
    .ok_or_else(invalid)?;
    let valid = receipt.actor_did == actor
        && task.transaction_id.as_deref() == Some(receipt.transaction_id.as_str())
        && task.terms_digest.as_deref() == Some(receipt.terms_digest.as_str())
        && valid_completion_evidence(task, receipt);
    if !valid {
        return Err(invalid());
    }
    entry(state, Mutation::Task(receipt), actor)
}

fn valid_completion_evidence(task: &Task, receipt: &TaskReceipt) -> bool {
    match receipt.action.as_str() {
        "task:complete" => {
            receipt.completion_evidence_digest.is_some()
                && task.completion_evidence_digest == receipt.completion_evidence_digest
        }
        _ => receipt.completion_evidence_digest.is_none(),
    }
}

fn escrow_entry(
    state: &State,
    task: &Task,
    escrow: &Escrow,
    receipt: &EscrowReceipt,
) -> Result<ChainEntry, String> {
    let actor = if receipt.action == "escrow:fund" {
        escrow.funder_did.as_deref()
    } else {
        escrow.release_authority_did.as_deref()
    }
    .ok_or_else(invalid)?;
    if !valid_escrow_binding(task, escrow, receipt, actor) {
        return Err(invalid());
    }
    entry(state, Mutation::Escrow(receipt), actor)
}

fn valid_escrow_binding(
    task: &Task,
    escrow: &Escrow,
    receipt: &EscrowReceipt,
    actor: &str,
) -> bool {
    receipt.actor_did == actor
        && receipt.task_id == task.task_id
        && escrow.transaction_id.as_deref() == Some(receipt.transaction_id.as_str())
        && escrow.terms_digest.as_deref() == Some(receipt.terms_digest.as_str())
        && escrow.network.as_deref() == Some(receipt.network.as_str())
        && escrow.amount_lamports == Some(receipt.amount_lamports)
        && escrow.release_policy.as_deref() == Some(receipt.release_policy.as_str())
}

fn entry(state: &State, mutation: Mutation<'_>, actor: &str) -> Result<ChainEntry, String> {
    let fields = mutation.fields();
    let authorization = state
        .authorization_receipts
        .iter()
        .find(|receipt| {
            receipt.actor_did == actor
                && receipt.action == fields.authorization_action
                && receipt.resource == fields.authorization_resource
                && receipt.decision == "allow"
        })
        .ok_or_else(invalid)?;
    Ok(ChainEntry {
        receipt_id: fields.receipt_id.to_owned(),
        receipt_digest: fields.receipt_digest,
        authorization_digest: digest::authorization(authorization),
        actor_did: actor.to_owned(),
        action: fields.action.to_owned(),
        resource_id: fields.resource_id.to_owned(),
        correlation_id: fields.correlation_id.to_owned(),
        idempotency_key: fields.idempotency_key.to_owned(),
        prior_state: fields.prior_state.to_owned(),
        resulting_state: fields.resulting_state.to_owned(),
    })
}

pub(super) fn require_unique(entries: &[ChainEntry]) -> Result<(), String> {
    let ids = entries
        .iter()
        .map(|entry| &entry.receipt_id)
        .collect::<BTreeSet<_>>();
    let keys = entries
        .iter()
        .map(|entry| (&entry.actor_did, &entry.idempotency_key))
        .collect::<BTreeSet<_>>();
    (ids.len() == entries.len() && keys.len() == entries.len())
        .then_some(())
        .ok_or_else(invalid)
}

pub(super) fn durable_receipts(entries: &[ChainEntry]) -> Vec<DurableReceipt> {
    entries
        .iter()
        .map(|entry| DurableReceipt {
            actor_did: entry.actor_did.clone(),
            action: entry.action.clone(),
            resource_id: entry.resource_id.clone(),
            resulting_state: entry.resulting_state.clone(),
            receipt_id: entry.receipt_id.clone(),
            receipt_digest: entry.receipt_digest.clone(),
        })
        .collect()
}
