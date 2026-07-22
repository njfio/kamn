#![allow(dead_code)]

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{intent, ESCROW};

pub(super) fn task_digest(value: &Value) -> String {
    record_digest(
        "kamn.service.task-receipt.v1",
        value,
        &[
            "receipt_id",
            "correlation_id",
            "idempotency_key",
            "actor_did",
            "task_id",
            "transaction_id",
            "action",
            "prior_state",
            "resulting_state",
            "terms_digest",
        ],
        true,
    )
}

pub(super) fn escrow_digest(value: &Value) -> String {
    record_digest(
        "kamn.service.escrow-receipt.v1",
        value,
        &[
            "receipt_id",
            "correlation_id",
            "idempotency_key",
            "actor_did",
            "escrow_id",
            "task_id",
            "transaction_id",
            "action",
            "prior_state",
            "resulting_state",
            "network",
            "amount_lamports",
            "terms_digest",
            "release_policy",
        ],
        false,
    )
}

fn authorization_digest(value: &Value) -> String {
    record_digest(
        "kamn.service.authorization-receipt.v1",
        value,
        &[
            "receipt_id",
            "correlation_id",
            "actor_did",
            "resource",
            "action",
            "role",
            "decision",
            "reason_code",
        ],
        false,
    )
}

fn record_digest(domain: &str, value: &Value, names: &[&str], completion: bool) -> String {
    let mut fields = vec![domain.to_owned()];
    fields.extend(names.iter().map(|name| text(&value[*name])));
    if completion {
        let evidence = value["completion_evidence_digest"].as_str();
        fields.push(if evidence.is_some() { "some" } else { "none" }.to_owned());
        fields.push(evidence.unwrap_or("").to_owned());
    }
    hash_fields(fields.iter().map(String::as_str))
}

pub(super) fn entry(
    value: &Value,
    auth: &Value,
    resource: &str,
    receipt_digest: String,
) -> Vec<String> {
    [
        text(&value["receipt_id"]),
        receipt_digest,
        authorization_digest(auth),
        text(&value["actor_did"]),
        text(&value["action"]),
        text(&value[resource]),
        text(&value["correlation_id"]),
        text(&value["idempotency_key"]),
        text(&value["prior_state"]),
        text(&value["resulting_state"]),
    ]
    .to_vec()
}

pub(super) fn settlement_entry() -> Vec<String> {
    let value = intent("unused");
    let digest = record_digest(
        "kamn.service.settlement-intent.v1",
        &value,
        &[
            "settlement_intent_id",
            "escrow_id",
            "actor_did",
            "idempotency_key",
            "amount_lamports",
            "network",
            "expected_signature",
            "signed_transaction_digest",
            "state",
        ],
        false,
    );
    vec![
        text(&value["settlement_intent_id"]),
        digest,
        String::new(),
        text(&value["actor_did"]),
        "settlement:confirmed".to_owned(),
        ESCROW.to_owned(),
        String::new(),
        text(&value["idempotency_key"]),
        "submitted".to_owned(),
        "confirmed".to_owned(),
    ]
}

fn text(value: &Value) -> String {
    value
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| value.to_string())
}

pub(super) fn hash_fields<'a>(fields: impl Iterator<Item = &'a str>) -> String {
    let mut hasher = Sha256::new();
    for field in fields {
        hasher.update((field.len() as u64).to_be_bytes());
        hasher.update(field.as_bytes());
    }
    format!("sha256:{:x}", hasher.finalize())
}
