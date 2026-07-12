use std::path::Path;
use std::process::Command;

use serde_json::Value;

use super::AgentTransactionDemoConfig;

const PI_ERROR: &str = "AGENT_TRANSACTION_PI_PREFLIGHT_FAILED";
const AGENT_ERROR: &str = "AGENT_TRANSACTION_AGENT_CONFIG_INVALID";
const DEVNET_ERROR: &str = "AGENT_TRANSACTION_DEVNET_CONFIG_INVALID";

/// Proves external files and current Pi OAuth/model readiness before execution.
pub fn validate_agent_transaction_preflight(
    config: &AgentTransactionDemoConfig,
) -> Result<(), String> {
    for path in &config.agent_key_files {
        require_nonempty_file(path, AGENT_ERROR)?;
    }
    validate_payer_keypair(config.solana_keypair_file.as_str())?;
    require_nonempty_file(config.pi_extension.as_str(), PI_ERROR)?;
    require_executable(config.local_node_binary.as_str())?;
    require_nonempty_file(config.mcp_binary.as_str(), AGENT_ERROR)?;
    validate_recipient(config.solana_recipient_pubkey.as_str())?;
    probe_pi_auth(config)
}

fn require_executable(path: &str) -> Result<(), String> {
    require_nonempty_file(path, AGENT_ERROR)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(path)
            .map_err(|_| format!("{AGENT_ERROR}: local node metadata unavailable"))?
            .permissions()
            .mode();
        if mode & 0o111 == 0 {
            return Err(format!("{AGENT_ERROR}: local node is not executable"));
        }
    }
    Ok(())
}

fn require_nonempty_file(path: &str, code: &str) -> Result<(), String> {
    let metadata = std::fs::metadata(path)
        .map_err(|_| format!("{code}: required external file is unavailable"))?;
    if metadata.is_file() && metadata.len() > 0 {
        return Ok(());
    }
    Err(format!("{code}: required external file is empty"))
}

fn validate_payer_keypair(path: &str) -> Result<(), String> {
    require_nonempty_file(path, DEVNET_ERROR)?;
    let raw = std::fs::read_to_string(path)
        .map_err(|_| format!("{DEVNET_ERROR}: payer keypair is unreadable"))?;
    let value: Value = serde_json::from_str(raw.as_str())
        .map_err(|_| format!("{DEVNET_ERROR}: payer keypair JSON is invalid"))?;
    let valid = value.as_array().is_some_and(|items| {
        items.len() == 64
            && items
                .iter()
                .all(|item| item.as_u64().is_some_and(|byte| byte <= u8::MAX as u64))
    });
    if valid {
        return Ok(());
    }
    Err(format!(
        "{DEVNET_ERROR}: payer keypair must contain 64 bytes"
    ))
}

fn validate_recipient(value: &str) -> Result<(), String> {
    let valid_length = (32..=44).contains(&value.len());
    let valid_chars = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() && !matches!(byte, b'0' | b'O' | b'I' | b'l'));
    if valid_length && valid_chars {
        return Ok(());
    }
    Err(format!("{DEVNET_ERROR}: recipient pubkey is invalid"))
}

fn probe_pi_auth(config: &AgentTransactionDemoConfig) -> Result<(), String> {
    let output = Command::new(Path::new(config.pi_binary.as_str()))
        .args([
            "--provider",
            config.pi_provider.as_str(),
            "--model",
            config.pi_model.as_str(),
            "--no-tools",
            "--no-session",
            "--no-extensions",
            "--no-skills",
            "--no-context-files",
            "--print",
            "Reply exactly KAMN_PI_PREFLIGHT_OK",
        ])
        .output()
        .map_err(|_| format!("{PI_ERROR}: Pi executable is unavailable"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if output.status.success() && stdout.contains("KAMN_PI_PREFLIGHT_OK") {
        return Ok(());
    }
    Err(format!("{PI_ERROR}: OAuth/model probe failed"))
}
