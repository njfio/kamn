use serde_json::Value;
use std::path::{Path, PathBuf};

use super::AgentTransactionDemoConfig;

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
