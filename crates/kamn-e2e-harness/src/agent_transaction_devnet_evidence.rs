use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use super::agent_transaction_evidence::AgentTransactionEvidencePaths;
use super::AgentTransactionDemoConfig;

const EVIDENCE_ERROR: &str = "AGENT_TRANSACTION_SETTLEMENT_INVALID";

pub(super) fn collect_actor_settlement_evidence(
    config: &AgentTransactionDemoConfig,
    paths: &AgentTransactionEvidencePaths,
) -> Result<PathBuf, String> {
    let actor = read_actor(paths.actors[0].as_str())?;
    let payer = payer_pubkey(config.solana_keypair_file.as_str())?;
    let confirmation = confirm(config, actor.signature.as_str())?;
    let balances = transaction_balances(&confirmation, payer.as_str(), config)?;
    let evidence = evidence_json(config, &actor, payer.as_str(), &balances);
    let path = Path::new(config.staging_root.as_str()).join("actor-devnet-evidence.json");
    std::fs::write(&path, evidence)
        .map_err(|error| settlement_error(format!("evidence write failed: {error}")))?;
    Ok(path)
}

struct ActorSettlement {
    signature: String,
    escrow_id: String,
    lamports: u64,
}

struct TransactionBalances {
    payer_before: u64,
    payer_after: u64,
    recipient_before: u64,
    recipient_after: u64,
}

fn read_actor(path: &str) -> Result<ActorSettlement, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| settlement_error(format!("actor read failed: {error}")))?;
    let value: Value = serde_json::from_str(raw.as_str())
        .map_err(|error| settlement_error(format!("actor JSON failed: {error}")))?;
    Ok(ActorSettlement {
        signature: string_field(&value, "settlement_tx_signature")?,
        escrow_id: string_field(&value, "escrow_id")?,
        lamports: u64_field(&value, "amount_lamports")?,
    })
}

fn payer_pubkey(keypair: &str) -> Result<String, String> {
    let output = Command::new("solana-keygen")
        .args(["pubkey", keypair])
        .output()
        .map_err(|error| settlement_error(format!("solana-keygen failed: {error}")))?;
    if !output.status.success() {
        return Err(settlement_error("solana-keygen rejected payer"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn confirm(config: &AgentTransactionDemoConfig, signature: &str) -> Result<Value, String> {
    let output = Command::new("solana")
        .args([
            "confirm",
            "--url",
            config.solana_rpc_url.as_str(),
            "--commitment",
            "finalized",
            "--verbose",
            "--output",
            "json",
            signature,
        ])
        .output()
        .map_err(|error| settlement_error(format!("solana confirm failed: {error}")))?;
    if !output.status.success() {
        return Err(settlement_error("Solana did not confirm actor settlement"));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| settlement_error(format!("confirmation JSON failed: {error}")))
}

fn transaction_balances(
    value: &Value,
    payer: &str,
    config: &AgentTransactionDemoConfig,
) -> Result<TransactionBalances, String> {
    require_finalized(value)?;
    let keys = value["transaction"]["message"]["accountKeys"]
        .as_array()
        .ok_or_else(|| settlement_error("confirmation account keys missing"))?;
    let payer_index = account_index(keys, payer)?;
    let recipient_index = account_index(keys, config.solana_recipient_pubkey.as_str())?;
    let pre = balance_array(value, "preBalances")?;
    let post = balance_array(value, "postBalances")?;
    let balances = balances(pre, post, payer_index, recipient_index)?;
    validate_movement(&balances, config.solana_lamports)?;
    Ok(balances)
}

fn require_finalized(value: &Value) -> Result<(), String> {
    if value["confirmationStatus"] == "finalized" && value["meta"]["err"].is_null() {
        return Ok(());
    }
    Err(settlement_error(
        "actor transaction is not finalized success",
    ))
}

fn account_index(keys: &[Value], expected: &str) -> Result<usize, String> {
    keys.iter()
        .position(|key| key.as_str() == Some(expected))
        .ok_or_else(|| settlement_error(format!("transaction account missing: {expected}")))
}

fn balance_array<'a>(value: &'a Value, field: &str) -> Result<&'a [Value], String> {
    value["meta"][field]
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| settlement_error(format!("confirmation {field} missing")))
}

fn balances(
    pre: &[Value],
    post: &[Value],
    payer: usize,
    recipient: usize,
) -> Result<TransactionBalances, String> {
    Ok(TransactionBalances {
        payer_before: indexed_balance(pre, payer)?,
        payer_after: indexed_balance(post, payer)?,
        recipient_before: indexed_balance(pre, recipient)?,
        recipient_after: indexed_balance(post, recipient)?,
    })
}

fn indexed_balance(values: &[Value], index: usize) -> Result<u64, String> {
    values
        .get(index)
        .and_then(Value::as_u64)
        .ok_or_else(|| settlement_error("transaction balance missing"))
}

fn validate_movement(found: &TransactionBalances, lamports: u64) -> Result<(), String> {
    let recipient_moved = found.recipient_after >= found.recipient_before.saturating_add(lamports);
    if recipient_moved && found.payer_after < found.payer_before {
        return Ok(());
    }
    Err(settlement_error(
        "actor transaction balance movement invalid",
    ))
}

fn evidence_json(
    config: &AgentTransactionDemoConfig,
    actor: &ActorSettlement,
    payer: &str,
    balances: &TransactionBalances,
) -> String {
    serde_json::json!({
        "network": "solana:devnet", "rpc_url": config.solana_rpc_url,
        "payer_pubkey": payer, "recipient_pubkey": config.solana_recipient_pubkey,
        "lamports": actor.lamports, "escrow_id": actor.escrow_id,
        "settlement_tx_signature": actor.signature, "settlement_commitment": "finalized",
        "payer_balance_before": balances.payer_before, "payer_balance_after": balances.payer_after,
        "recipient_balance_before": balances.recipient_before,
        "recipient_balance_after": balances.recipient_after,
        "persisted_settlement_tx_signature": actor.signature,
    })
    .to_string()
}

fn string_field(value: &Value, field: &str) -> Result<String, String> {
    value[field]
        .as_str()
        .filter(|found| !found.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| settlement_error(format!("actor {field} missing")))
}

fn u64_field(value: &Value, field: &str) -> Result<u64, String> {
    value[field]
        .as_u64()
        .filter(|found| *found > 0)
        .ok_or_else(|| settlement_error(format!("actor {field} missing")))
}

fn settlement_error(message: impl AsRef<str>) -> String {
    format!("{EVIDENCE_ERROR}: {}", message.as_ref())
}
