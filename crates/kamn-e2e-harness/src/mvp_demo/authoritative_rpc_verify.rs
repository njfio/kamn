use serde_json::Value;
use std::path::Path;

use super::artifact_digest::sha256_hex;
use super::settlement_evidence_artifact::SettlementEvidenceArtifact;

pub(super) fn validate_authoritative_rpc(
    report: &Value,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    if report["artifacts"]["solana_confirmation_response"].is_null() {
        return Err(invalid());
    }
    let path = report["artifacts"]["solana_confirmation_response"]
        .as_str()
        .ok_or_else(invalid)?;
    let raw = std::fs::read_to_string(Path::new(path)).map_err(|_| invalid())?;
    let expected = evidence
        .authoritative_rpc_digest
        .as_deref()
        .ok_or_else(invalid)?;
    if format!("sha256:{}", sha256_hex(raw.as_str())) != expected {
        return Err(invalid());
    }
    let response: Value = serde_json::from_str(raw.as_str()).map_err(|_| invalid())?;
    validate_confirmation(&response, evidence)
}

fn validate_confirmation(
    response: &Value,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    if response["confirmationStatus"] != "finalized" || !response["meta"]["err"].is_null() {
        return Err(invalid());
    }
    require_signature(response, evidence.settlement_tx_signature.as_str())?;
    let keys = array(response, &["transaction", "message", "accountKeys"])?;
    let payer = account_index(keys, evidence.payer_pubkey.as_str())?;
    let recipient = account_index(keys, evidence.recipient_pubkey.as_str())?;
    validate_response_balances(response, evidence, payer, recipient)
}

fn validate_response_balances(
    response: &Value,
    evidence: &SettlementEvidenceArtifact,
    payer: usize,
    recipient: usize,
) -> Result<(), String> {
    let pre = array(response, &["meta", "preBalances"])?;
    let post = array(response, &["meta", "postBalances"])?;
    for (values, index, expected) in [
        (pre, payer, evidence.payer_balance_before),
        (post, payer, evidence.payer_balance_after),
        (pre, recipient, evidence.recipient_balance_before),
        (post, recipient, evidence.recipient_balance_after),
    ] {
        if values.get(index).and_then(Value::as_u64) != Some(expected) {
            return Err(invalid());
        }
    }
    Ok(())
}

fn require_signature(response: &Value, expected: &str) -> Result<(), String> {
    let signatures = array(response, &["transaction", "signatures"])?;
    if signatures.first().and_then(Value::as_str) == Some(expected) {
        return Ok(());
    }
    Err(invalid())
}

fn account_index(keys: &[Value], expected: &str) -> Result<usize, String> {
    keys.iter()
        .position(|value| value.as_str() == Some(expected))
        .ok_or_else(invalid)
}

fn array<'a>(value: &'a Value, path: &[&str]) -> Result<&'a [Value], String> {
    path.iter()
        .fold(value, |current, key| &current[*key])
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(invalid)
}

fn invalid() -> String {
    "SETTLEMENT_EVIDENCE_INVALID".to_owned()
}
