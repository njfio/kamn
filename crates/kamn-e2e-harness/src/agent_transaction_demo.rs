use std::collections::{BTreeMap, HashSet};

const CONFIG_ERROR: &str = "AGENT_TRANSACTION_DEVNET_CONFIG_INVALID";

/// Validated configuration for the canonical three-agent transaction demo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTransactionDemoConfig {
    /// Agent runtime driver. The MVP supports only `pi`.
    pub agent_driver: String,
    /// Settlement mode. The canonical transaction requires `required`.
    pub devnet_mode: String,
    /// Solana devnet RPC URL.
    pub solana_rpc_url: String,
    /// Three external KAMN agent key-file paths.
    pub agent_key_files: [String; 3],
    /// External Solana payer keypair path.
    pub solana_keypair_file: String,
    /// Solana devnet recipient pubkey.
    pub solana_recipient_pubkey: String,
    /// Positive transfer amount in lamports.
    pub solana_lamports: u64,
    /// Required settlement commitment.
    pub solana_commitment: String,
}

/// Parses and validates canonical demo configuration without performing work.
pub fn parse_agent_transaction_demo_config(
    env: &BTreeMap<String, String>,
) -> Result<AgentTransactionDemoConfig, String> {
    let config = AgentTransactionDemoConfig {
        agent_driver: required(env, "KAMN_MVP_AGENT_DRIVER")?,
        devnet_mode: required(env, "KAMN_MVP_DEVNET_MODE")?,
        solana_rpc_url: required(env, "KAMN_MVP_SOLANA_RPC_URL")?,
        agent_key_files: agent_keys(env)?,
        solana_keypair_file: required(env, "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE")?,
        solana_recipient_pubkey: required(
            env,
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY",
        )?,
        solana_lamports: positive_u64(env, "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS")?,
        solana_commitment: required(env, "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT")?,
    };
    validate_config(&config)?;
    Ok(config)
}

fn agent_keys(env: &BTreeMap<String, String>) -> Result<[String; 3], String> {
    Ok([
        required(env, "KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE")?,
        required(env, "KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE")?,
        required(env, "KAMN_MVP_LIVE_MCP_AGENT_C_KEY_FILE")?,
    ])
}

fn required(env: &BTreeMap<String, String>, name: &str) -> Result<String, String> {
    env.get(name)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| config_error(format!("missing {name}")))
}

fn positive_u64(env: &BTreeMap<String, String>, name: &str) -> Result<u64, String> {
    let value = required(env, name)?;
    value
        .parse::<u64>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .ok_or_else(|| config_error(format!("{name} must be a positive integer")))
}

fn validate_config(config: &AgentTransactionDemoConfig) -> Result<(), String> {
    if config.agent_driver != "pi" {
        return Err("AGENT_TRANSACTION_DRIVER_INVALID: expected pi".to_owned());
    }
    if config.devnet_mode != "required"
        || !config.solana_rpc_url.starts_with("https://")
        || !config.solana_rpc_url.contains("devnet")
        || config.solana_commitment != "finalized"
    {
        return Err(config_error(
            "required finalized Solana devnet configuration",
        ));
    }
    require_distinct_agent_keys(&config.agent_key_files)
}

fn require_distinct_agent_keys(paths: &[String; 3]) -> Result<(), String> {
    let unique = paths.iter().collect::<HashSet<_>>();
    if unique.len() == paths.len() {
        return Ok(());
    }
    Err("AGENT_TRANSACTION_AGENT_CONFIG_INVALID: agent key paths must be distinct".to_owned())
}

fn config_error(message: impl AsRef<str>) -> String {
    format!("{CONFIG_ERROR}: {}", message.as_ref())
}
