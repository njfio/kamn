use serde_json::Value;

use super::agent_transaction_receipt_chain_model::DurableReceipt;

const ERROR: &str = "PI_SERVICE_AUTHORITY_MISMATCH";

pub(super) fn validate(
    paths: &[String; 3],
    commitment: &str,
    durable: &[DurableReceipt],
) -> Result<(), String> {
    let receipts = paths
        .iter()
        .map(|path| read_actor(path, commitment))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let valid = receipts.len() == durable.len()
        && receipts
            .iter()
            .all(|receipt| durable.iter().any(|stored| matches(receipt, stored)));
    valid.then_some(()).ok_or_else(invalid)
}

fn read_actor(path: &str, commitment: &str) -> Result<Vec<Value>, String> {
    let raw = std::fs::read_to_string(path).map_err(|_| invalid())?;
    let actor: Value = serde_json::from_str(raw.as_str()).map_err(|_| invalid())?;
    if actor["receipt_chain_commitment"].as_str() != Some(commitment) {
        return Err(invalid());
    }
    actor["service_receipts"]
        .as_array()
        .cloned()
        .ok_or_else(invalid)
}

fn matches(actor: &Value, stored: &DurableReceipt) -> bool {
    string(actor, "actor_did") == stored.actor_did
        && string(actor, "action") == stored.action
        && string(actor, "resource_id") == stored.resource_id
        && string(actor, "resulting_state") == stored.resulting_state
        && string(actor, "service_receipt_id") == stored.receipt_id
        && string(actor, "service_receipt_digest") == stored.receipt_digest
}

fn string<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field].as_str().unwrap_or_default()
}

fn invalid() -> String {
    ERROR.to_owned()
}
