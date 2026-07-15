use serde_json::Value;
use std::path::{Path, PathBuf};

use super::AgentTransactionDemoConfig;

pub(super) struct ConfirmedTransfer {
    pub(super) payer: String,
    pub(super) payer_before: u64,
    pub(super) payer_after: u64,
    pub(super) recipient_before: u64,
    pub(super) recipient_after: u64,
    pub(super) fee_lamports: u64,
    pub(super) artifact_path: PathBuf,
}

pub(super) fn confirm_transfer(
    config: &AgentTransactionDemoConfig,
    signature: &str,
) -> Result<ConfirmedTransfer, String> {
    let payer = payer_pubkey(config.solana_keypair_file.as_str())?;
    let value = confirmation(config, signature)?;
    require_finalized(&value, signature)?;
    let artifact_path = write_confirmation(config, &value)?;
    let keys = array(&value, &["transaction", "message", "accountKeys"])?;
    let payer_index = account_index(keys, payer.as_str())?;
    let recipient_index = account_index(keys, config.solana_recipient_pubkey.as_str())?;
    let transfer = transfer(&value, payer, payer_index, recipient_index, artifact_path)?;
    validate_movement(&transfer, config.solana_lamports)?;
    Ok(transfer)
}

pub(super) fn payer_pubkey(keypair: &str) -> Result<String, String> {
    let output = std::process::Command::new("solana-keygen")
        .args(["pubkey", keypair])
        .output()
        .map_err(|error| format_error("payer lookup", error))?;
    if !output.status.success() {
        return Err(format_error("payer lookup", "keypair rejected"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub(super) fn write_confirmation(
    config: &AgentTransactionDemoConfig,
    value: &Value,
) -> Result<PathBuf, String> {
    let path = Path::new(config.staging_root.as_str()).join("actor-solana-confirmation.json");
    let raw = serde_json::to_vec(value).map_err(|error| format_error("encode", error))?;
    std::fs::write(&path, raw).map_err(|error| format_error("write", error))?;
    Ok(path)
}

fn format_error(action: &str, error: impl std::fmt::Display) -> String {
    format!("AGENT_TRANSACTION_SETTLEMENT_INVALID: confirmation {action} failed: {error}")
}

fn confirmation(config: &AgentTransactionDemoConfig, signature: &str) -> Result<Value, String> {
    let output = std::process::Command::new("solana")
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
        .map_err(|error| format_error("Solana RPC", error))?;
    if !output.status.success() {
        return Err(format_error("Solana RPC", "transaction not confirmed"));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| format_error("JSON", error))
}

fn require_finalized(value: &Value, signature: &str) -> Result<(), String> {
    let found = array(value, &["transaction", "signatures"])?
        .first()
        .and_then(Value::as_str);
    if value["confirmationStatus"] == "finalized"
        && value["meta"]["err"].is_null()
        && found == Some(signature)
    {
        return Ok(());
    }
    Err(format_error("finality", "transaction evidence mismatch"))
}

fn transfer(
    value: &Value,
    payer: String,
    payer_index: usize,
    recipient_index: usize,
    artifact_path: PathBuf,
) -> Result<ConfirmedTransfer, String> {
    let pre = array(value, &["meta", "preBalances"])?;
    let post = array(value, &["meta", "postBalances"])?;
    Ok(ConfirmedTransfer {
        payer,
        payer_before: balance(pre, payer_index)?,
        payer_after: balance(post, payer_index)?,
        recipient_before: balance(pre, recipient_index)?,
        recipient_after: balance(post, recipient_index)?,
        fee_lamports: value["meta"]["fee"]
            .as_u64()
            .ok_or_else(|| format_error("fee", "missing"))?,
        artifact_path,
    })
}

fn validate_movement(found: &ConfirmedTransfer, lamports: u64) -> Result<(), String> {
    let payer = found.payer_before.checked_sub(found.payer_after);
    let recipient = found.recipient_after.checked_sub(found.recipient_before);
    if payer == lamports.checked_add(found.fee_lamports) && recipient == Some(lamports) {
        return Ok(());
    }
    Err(format_error("balance movement", "amount or fee mismatch"))
}

fn array<'a>(value: &'a Value, path: &[&str]) -> Result<&'a [Value], String> {
    path.iter()
        .fold(value, |current, field| &current[*field])
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format_error("array", path.join(".")))
}

fn account_index(keys: &[Value], expected: &str) -> Result<usize, String> {
    keys.iter()
        .position(|key| key.as_str() == Some(expected))
        .ok_or_else(|| format_error("account", expected))
}

fn balance(values: &[Value], index: usize) -> Result<u64, String> {
    values
        .get(index)
        .and_then(Value::as_u64)
        .ok_or_else(|| format_error("balance", index))
}
