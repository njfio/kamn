#![allow(dead_code)]

use serde_json::{json, Value};

#[path = "service_authority_digest_fixture.rs"]
mod digest_fixture;
use digest_fixture::{entry, escrow_digest, hash_fields, settlement_entry, task_digest};

pub(crate) const TASK: &str = "task-local-bound-7086";
pub(crate) const TRANSACTION: &str = "transaction-live-7086";
pub(crate) const ESCROW: &str = "escrow-local-bound-7086";
pub(crate) const SIGNATURE: &str = "devnet-signature-111";
const TERMS: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const EVIDENCE: &str = "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

pub(crate) fn state(recipient: &str) -> Value {
    json!({
        "schema_version": "kamn.runtime.service-api-message-store.v4",
        "tasks": {TASK: task()},
        "escrows": {ESCROW: escrow()},
        "authorization_receipts": authorizations(),
        "task_transition_receipts": task_receipts(),
        "escrow_transition_receipts": escrow_receipts(),
        "settlement_intents": {ESCROW: intent(recipient)},
    })
}

pub(crate) fn actor_receipts(role: &str) -> Vec<Value> {
    let all = mutation_receipts();
    match role {
        "agent_a" => vec![
            all[0].clone(),
            all[2].clone(),
            all[4].clone(),
            settlement_receipt(),
        ],
        "agent_b" => vec![all[1].clone(), all[3].clone()],
        _ => Vec::new(),
    }
}

fn settlement_receipt() -> Value {
    let fields = settlement_entry();
    json!({
        "actor_did": fields[3], "tool": "release_escrow", "action": fields[4],
        "resource_id": fields[5], "resulting_state": fields[9],
        "service_receipt_id": fields[0], "service_receipt_digest": fields[1],
    })
}

pub(crate) fn commitment() -> String {
    let task = task_receipts();
    let escrow = escrow_receipts();
    let auth = authorizations();
    let mutations = [
        entry(&task[0], &auth[0], "task_id", task_digest(&task[0])),
        entry(&task[1], &auth[1], "task_id", task_digest(&task[1])),
        entry(&escrow[0], &auth[2], "escrow_id", escrow_digest(&escrow[0])),
        entry(&task[2], &auth[3], "task_id", task_digest(&task[2])),
        entry(&escrow[1], &auth[4], "escrow_id", escrow_digest(&escrow[1])),
        settlement_entry(),
    ];
    let mut fields = vec!["kamn.service.receipt-chain.v1".to_owned(), "6".to_owned()];
    for entry in mutations {
        fields.extend(entry);
    }
    hash_fields(fields.iter().map(String::as_str))
}

fn task() -> Value {
    json!({"task_id": TASK, "state": "completed", "creator_did": "kamn:did:a",
        "provider_did": "kamn:did:b", "transaction_id": TRANSACTION,
        "terms_digest": TERMS, "completion_evidence_digest": EVIDENCE})
}

fn escrow() -> Value {
    json!({"escrow_id": ESCROW, "state": "released", "task_id": TASK,
        "transaction_id": TRANSACTION, "funder_did": "kamn:did:a",
        "amount_lamports": 1000000, "network": "solana-devnet", "terms_digest": TERMS,
        "release_authority_did": "kamn:did:a", "release_policy": "creator-authorized",
        "settlement_receipt_hash": SIGNATURE, "settlement_tx_signature": SIGNATURE,
        "settlement_network": "solana:devnet", "settlement_commitment": "finalized"})
}

pub(super) fn authorizations() -> Vec<Value> {
    let task = format!("task:{TASK}");
    let escrow = format!("escrow:{ESCROW}");
    vec![
        authorization(
            "01",
            "kamn:did:a",
            "transaction:new",
            "task:create",
            "creator",
        ),
        authorization("02", "kamn:did:b", &task, "task:accept", "provider"),
        authorization("03", "kamn:did:a", &task, "escrow:fund", "creator"),
        authorization("04", "kamn:did:b", &task, "task:complete", "provider"),
        authorization("05", "kamn:did:a", &escrow, "escrow:release", "creator"),
    ]
}

fn authorization(id: &str, actor: &str, resource: &str, action: &str, role: &str) -> Value {
    json!({"receipt_id": format!("authorization-{id}"), "correlation_id": format!("correlation-{id}"),
        "actor_did": actor, "resource": resource, "action": action, "role": role,
        "decision": "allow", "reason_code": "authorized"})
}

pub(super) fn task_receipts() -> Vec<Value> {
    vec![
        task_receipt("01", "kamn:did:a", "task:create", "none", "submitted", None),
        task_receipt(
            "03",
            "kamn:did:b",
            "task:accept",
            "submitted",
            "accepted",
            None,
        ),
        task_receipt(
            "04",
            "kamn:did:b",
            "task:complete",
            "accepted",
            "completed",
            Some(EVIDENCE),
        ),
    ]
}

fn task_receipt(
    id: &str,
    actor: &str,
    action: &str,
    before: &str,
    after: &str,
    evidence: Option<&str>,
) -> Value {
    json!({"receipt_id": format!("service-receipt-{id}"), "correlation_id": format!("correlation-{id}"),
        "idempotency_key": format!("idempotency-{id}"), "actor_did": actor, "task_id": TASK,
        "transaction_id": TRANSACTION, "action": action, "prior_state": before,
        "resulting_state": after, "terms_digest": TERMS, "completion_evidence_digest": evidence})
}

pub(super) fn escrow_receipts() -> Vec<Value> {
    vec![
        escrow_receipt("02", "escrow:fund", "unfunded", "funded"),
        escrow_receipt(
            "05",
            "escrow:release-authorize",
            "funded",
            "release-authorized",
        ),
    ]
}

fn escrow_receipt(id: &str, action: &str, before: &str, after: &str) -> Value {
    json!({"receipt_id": format!("service-receipt-{id}"), "correlation_id": format!("correlation-{id}"),
        "idempotency_key": format!("idempotency-{id}"), "actor_did": "kamn:did:a", "escrow_id": ESCROW,
        "task_id": TASK, "transaction_id": TRANSACTION, "action": action, "prior_state": before,
        "resulting_state": after, "network": "solana-devnet", "amount_lamports": 1000000,
        "terms_digest": TERMS, "release_policy": "creator-authorized"})
}

pub(super) fn intent(recipient: &str) -> Value {
    json!({"settlement_intent_id": "intent-local-bound-7086", "escrow_id": ESCROW,
        "actor_did": "kamn:did:a", "idempotency_key": "release-local-bound-7086",
        "recipient_pubkey": recipient, "amount_lamports": 1000000, "network": "solana:devnet",
        "expected_signature": SIGNATURE, "signed_transaction_digest": format!("sha256:{}", "b".repeat(64)),
        "signed_transaction_json": "signed-transaction-secret", "state": "confirmed", "submission_attempt_count": 1})
}

fn mutation_receipts() -> Vec<Value> {
    let tasks = task_receipts();
    let escrows = escrow_receipts();
    [(&tasks[0], "create_task", task_digest(&tasks[0])), (&tasks[1], "accept_task", task_digest(&tasks[1])),
        (&escrows[0], "fund_escrow", escrow_digest(&escrows[0])), (&tasks[2], "complete_task", task_digest(&tasks[2])),
        (&escrows[1], "release_escrow", escrow_digest(&escrows[1]))]
        .into_iter().map(|(receipt, tool, digest)| {
            let resource = if receipt["action"].as_str().is_some_and(|value| value.starts_with("task:")) {
                &receipt["task_id"]
            } else {
                &receipt["escrow_id"]
            };
            json!({"actor_did": receipt["actor_did"], "tool": tool,
                "action": receipt["action"], "resource_id": resource,
                "resulting_state": receipt["resulting_state"], "service_receipt_id": receipt["receipt_id"],
                "service_receipt_digest": digest})
        }).collect()
}
