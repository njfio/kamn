use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct State {
    pub(super) schema_version: String,
    #[serde(default)]
    pub(super) tasks: BTreeMap<String, Task>,
    #[serde(default)]
    pub(super) escrows: BTreeMap<String, Escrow>,
    #[serde(default)]
    pub(super) authorization_receipts: Vec<AuthorizationReceipt>,
    #[serde(default)]
    pub(super) task_transition_receipts: Vec<TaskReceipt>,
    #[serde(default)]
    pub(super) escrow_transition_receipts: Vec<EscrowReceipt>,
    #[serde(default)]
    pub(super) settlement_intents: BTreeMap<String, SettlementIntent>,
}

#[derive(Deserialize)]
pub(super) struct Task {
    pub(super) task_id: String,
    pub(super) state: String,
    pub(super) creator_did: Option<String>,
    pub(super) provider_did: Option<String>,
    pub(super) transaction_id: Option<String>,
    pub(super) terms_digest: Option<String>,
    pub(super) completion_evidence_digest: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct Escrow {
    pub(super) escrow_id: String,
    pub(super) state: String,
    pub(super) task_id: Option<String>,
    pub(super) transaction_id: Option<String>,
    pub(super) funder_did: Option<String>,
    pub(super) amount_lamports: Option<u64>,
    pub(super) network: Option<String>,
    pub(super) terms_digest: Option<String>,
    pub(super) release_authority_did: Option<String>,
    pub(super) release_policy: Option<String>,
    pub(super) settlement_tx_signature: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct AuthorizationReceipt {
    pub(super) receipt_id: String,
    pub(super) correlation_id: String,
    pub(super) actor_did: String,
    pub(super) resource: String,
    pub(super) action: String,
    pub(super) role: String,
    pub(super) decision: String,
    pub(super) reason_code: String,
}

#[derive(Deserialize)]
pub(super) struct TaskReceipt {
    pub(super) receipt_id: String,
    pub(super) correlation_id: String,
    pub(super) idempotency_key: String,
    pub(super) actor_did: String,
    pub(super) task_id: String,
    pub(super) transaction_id: String,
    pub(super) action: String,
    pub(super) prior_state: String,
    pub(super) resulting_state: String,
    pub(super) terms_digest: String,
    pub(super) completion_evidence_digest: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct EscrowReceipt {
    pub(super) receipt_id: String,
    pub(super) correlation_id: String,
    pub(super) idempotency_key: String,
    pub(super) actor_did: String,
    pub(super) escrow_id: String,
    pub(super) task_id: String,
    pub(super) transaction_id: String,
    pub(super) action: String,
    pub(super) prior_state: String,
    pub(super) resulting_state: String,
    pub(super) network: String,
    pub(super) amount_lamports: u64,
    pub(super) terms_digest: String,
    pub(super) release_policy: String,
}

#[derive(Deserialize)]
pub(super) struct SettlementIntent {
    pub(super) settlement_intent_id: String,
    pub(super) escrow_id: String,
    pub(super) actor_did: String,
    pub(super) idempotency_key: String,
    pub(super) amount_lamports: u64,
    pub(super) network: String,
    pub(super) expected_signature: String,
    pub(super) signed_transaction_digest: String,
    pub(super) state: String,
}

pub(super) struct ChainEntry {
    pub(super) receipt_id: String,
    pub(super) receipt_digest: String,
    pub(super) authorization_digest: String,
    pub(super) actor_did: String,
    pub(super) action: String,
    pub(super) resource_id: String,
    pub(super) correlation_id: String,
    pub(super) idempotency_key: String,
    pub(super) prior_state: String,
    pub(super) resulting_state: String,
}

#[derive(Clone)]
pub(super) struct DurableReceipt {
    pub(super) actor_did: String,
    pub(super) action: String,
    pub(super) resource_id: String,
    pub(super) resulting_state: String,
    pub(super) receipt_id: String,
    pub(super) receipt_digest: String,
}

pub(super) struct ReceiptChainEvidence {
    pub(super) commitment: String,
    pub(super) receipts: Vec<DurableReceipt>,
}
