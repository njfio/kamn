use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

pub(super) struct ExpectedSettlement<'a> {
    pub(super) task_id: &'a str,
    pub(super) transaction_id: &'a str,
    pub(super) escrow_id: &'a str,
    pub(super) signature: &'a str,
    pub(super) recipient: &'a str,
    pub(super) amount: u64,
}

pub(super) struct PersistedSettlement {
    pub(super) transaction_id: String,
    pub(super) terms_digest: String,
    pub(super) receipt_hash: String,
    pub(super) state_digest: String,
    pub(super) intent_digest: String,
}

pub(super) fn read_persisted_settlement(
    state_file: &Path,
    expected: &ExpectedSettlement<'_>,
) -> Result<PersistedSettlement, String> {
    let raw = std::fs::read_to_string(state_file)
        .map_err(|error| format!("persisted settlement state read failed: {error}"))?;
    let state: Value = serde_json::from_str(raw.as_str())
        .map_err(|error| format!("persisted settlement state JSON invalid: {error}"))?;
    let task = keyed(&state, "tasks", expected.task_id)?;
    let escrow = keyed(&state, "escrows", expected.escrow_id)?;
    let intent = keyed(&state, "settlement_intents", expected.escrow_id)?;
    validate_task(task, expected)?;
    validate_escrow(escrow, expected)?;
    validate_intent(intent, expected)?;
    build_persisted_settlement(raw.as_str(), task, escrow, intent)
}

fn build_persisted_settlement(
    raw: &str,
    task: &Value,
    escrow: &Value,
    intent: &Value,
) -> Result<PersistedSettlement, String> {
    let terms_digest = string(escrow, "terms_digest")?;
    if string(task, "terms_digest")? != terms_digest {
        return Err("persisted settlement terms_digest mismatch".to_owned());
    }
    Ok(PersistedSettlement {
        transaction_id: string(escrow, "transaction_id")?,
        terms_digest,
        receipt_hash: string(escrow, "settlement_receipt_hash")?,
        state_digest: digest(raw),
        intent_digest: digest(&serde_json::to_string(intent).map_err(json_error)?),
    })
}

fn keyed<'a>(state: &'a Value, collection: &str, id: &str) -> Result<&'a Value, String> {
    state
        .get(collection)
        .and_then(|value| value.get(id))
        .ok_or_else(|| format!("persisted settlement {collection} record missing: {id}"))
}

fn validate_task(task: &Value, expected: &ExpectedSettlement<'_>) -> Result<(), String> {
    require_string(task, "task_id", expected.task_id)?;
    require_string(task, "transaction_id", expected.transaction_id)?;
    require_string(task, "state", "completed")?;
    require_digest(task, "terms_digest")
}

fn validate_escrow(escrow: &Value, expected: &ExpectedSettlement<'_>) -> Result<(), String> {
    for (field, value) in [
        ("escrow_id", expected.escrow_id),
        ("task_id", expected.task_id),
        ("transaction_id", expected.transaction_id),
        ("state", "released"),
        ("network", "solana-devnet"),
        ("settlement_network", "solana:devnet"),
        ("settlement_commitment", "finalized"),
        ("settlement_tx_signature", expected.signature),
        ("settlement_receipt_hash", expected.signature),
    ] {
        require_string(escrow, field, value)?;
    }
    require_u64(escrow, "amount_lamports", expected.amount)?;
    require_digest(escrow, "terms_digest")
}

fn validate_intent(intent: &Value, expected: &ExpectedSettlement<'_>) -> Result<(), String> {
    require_string(intent, "escrow_id", expected.escrow_id)?;
    require_string(intent, "state", "confirmed")?;
    require_string(intent, "network", "solana:devnet")?;
    require_string(intent, "expected_signature", expected.signature)?;
    require_named_string(intent, "recipient_pubkey", expected.recipient, "recipient")?;
    require_u64(intent, "amount_lamports", expected.amount)?;
    require_u64(intent, "submission_attempt_count", 1)
}

fn require_named_string(
    value: &Value,
    field: &str,
    expected: &str,
    name: &str,
) -> Result<(), String> {
    if value.get(field).and_then(Value::as_str) == Some(expected) {
        return Ok(());
    }
    Err(format!("persisted settlement {name} mismatch"))
}

fn require_string(value: &Value, field: &str, expected: &str) -> Result<(), String> {
    require_named_string(value, field, expected, field)
}

fn require_u64(value: &Value, field: &str, expected: u64) -> Result<(), String> {
    if value.get(field).and_then(Value::as_u64) == Some(expected) {
        return Ok(());
    }
    Err(format!("persisted settlement {field} mismatch"))
}

fn require_digest(value: &Value, field: &str) -> Result<(), String> {
    let found = value.get(field).and_then(Value::as_str).unwrap_or_default();
    if found.len() == 64 && found.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(());
    }
    Err(format!("persisted settlement {field} invalid"))
}

fn string(value: &Value, field: &str) -> Result<String, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("persisted settlement {field} missing"))
}

fn digest(value: &str) -> String {
    format!("sha256:{:x}", Sha256::digest(value.as_bytes()))
}

fn json_error(error: serde_json::Error) -> String {
    format!("persisted settlement intent JSON invalid: {error}")
}
