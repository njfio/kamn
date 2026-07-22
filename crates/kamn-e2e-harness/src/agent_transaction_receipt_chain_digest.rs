use sha2::{Digest, Sha256};

use super::agent_transaction_receipt_chain_model::{
    AuthorizationReceipt, ChainEntry, EscrowReceipt, SettlementIntent, TaskReceipt,
};

pub(super) fn task(receipt: &TaskReceipt) -> String {
    let marker = if receipt.completion_evidence_digest.is_some() {
        "some"
    } else {
        "none"
    };
    digest(
        "kamn.service.task-receipt.v1",
        &[
            &receipt.receipt_id,
            &receipt.correlation_id,
            &receipt.idempotency_key,
            &receipt.actor_did,
            &receipt.task_id,
            &receipt.transaction_id,
            &receipt.action,
            &receipt.prior_state,
            &receipt.resulting_state,
            &receipt.terms_digest,
            marker,
            receipt.completion_evidence_digest.as_deref().unwrap_or(""),
        ],
    )
}

pub(super) fn escrow(receipt: &EscrowReceipt) -> String {
    let amount = receipt.amount_lamports.to_string();
    digest(
        "kamn.service.escrow-receipt.v1",
        &[
            &receipt.receipt_id,
            &receipt.correlation_id,
            &receipt.idempotency_key,
            &receipt.actor_did,
            &receipt.escrow_id,
            &receipt.task_id,
            &receipt.transaction_id,
            &receipt.action,
            &receipt.prior_state,
            &receipt.resulting_state,
            &receipt.network,
            &amount,
            &receipt.terms_digest,
            &receipt.release_policy,
        ],
    )
}

pub(super) fn authorization(receipt: &AuthorizationReceipt) -> String {
    digest(
        "kamn.service.authorization-receipt.v1",
        &[
            &receipt.receipt_id,
            &receipt.correlation_id,
            &receipt.actor_did,
            &receipt.resource,
            &receipt.action,
            &receipt.role,
            &receipt.decision,
            &receipt.reason_code,
        ],
    )
}

pub(super) fn settlement(intent: &SettlementIntent) -> String {
    let amount = intent.amount_lamports.to_string();
    digest(
        "kamn.service.settlement-intent.v1",
        &[
            &intent.settlement_intent_id,
            &intent.escrow_id,
            &intent.actor_did,
            &intent.idempotency_key,
            &amount,
            &intent.network,
            &intent.expected_signature,
            &intent.signed_transaction_digest,
            &intent.state,
        ],
    )
}

pub(super) fn chain(entries: &[ChainEntry]) -> String {
    let mut hasher = Sha256::new();
    append(&mut hasher, "kamn.service.receipt-chain.v1");
    append(&mut hasher, entries.len().to_string().as_str());
    for entry in entries {
        append_entry(&mut hasher, entry);
    }
    finish(hasher)
}

fn append_entry(hasher: &mut Sha256, entry: &ChainEntry) {
    for value in [
        &entry.receipt_id,
        &entry.receipt_digest,
        &entry.authorization_digest,
        &entry.actor_did,
        &entry.action,
        &entry.resource_id,
        &entry.correlation_id,
        &entry.idempotency_key,
        &entry.prior_state,
        &entry.resulting_state,
    ] {
        append(hasher, value);
    }
}

fn digest(domain: &str, fields: &[&str]) -> String {
    let mut hasher = Sha256::new();
    append(&mut hasher, domain);
    fields.iter().for_each(|field| append(&mut hasher, field));
    finish(hasher)
}

fn append(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

fn finish(hasher: Sha256) -> String {
    format!("sha256:{:x}", hasher.finalize())
}
