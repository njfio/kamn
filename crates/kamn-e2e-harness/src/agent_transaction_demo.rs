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
    /// Pi executable path or command name.
    pub pi_binary: String,
    /// Pi OAuth-backed provider name.
    pub pi_provider: String,
    /// Pi model identifier.
    pub pi_model: String,
    /// Explicit project-local Pi extension path.
    pub pi_extension: String,
    /// Local KAMN node executable path.
    pub local_node_binary: String,
    /// Local KAMN MCP server executable path exposed to Pi tools.
    pub mcp_binary: String,
    /// Local KAMN service API endpoint used by the MCP server.
    pub mcp_endpoint: String,
    /// Canonical proof output root.
    pub output_root: String,
    /// Maximum wait for one Pi RPC phase.
    pub rpc_timeout_ms: u64,
    /// Non-proof staging root for coordination and actor artifacts.
    pub staging_root: String,
    /// Optional devnet settlement command override at the external test boundary.
    pub devnet_settlement_command: Option<Vec<String>>,
    /// Optional localhost proof command override at the external test boundary.
    pub localhost_signed_demo_command: Option<Vec<String>>,
    /// Optional service API proof command override at the external test boundary.
    pub service_api_vertical_slice_command: Option<Vec<String>>,
    /// Optional websocket proof command override at the external test boundary.
    pub service_api_websocket_command: Option<Vec<String>>,
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
        pi_binary: optional(env, "KAMN_MVP_PI_BINARY", "pi"),
        pi_provider: optional(env, "KAMN_MVP_PI_PROVIDER", "openai-codex"),
        pi_model: optional(env, "KAMN_MVP_PI_MODEL", "gpt-5.5"),
        pi_extension: optional(
            env,
            "KAMN_MVP_PI_EXTENSION",
            ".pi/extensions/kamn-mvp/index.ts",
        ),
        local_node_binary: optional(env, "KAMN_MVP_LOCAL_NODE_BINARY", "target/debug/kamn-node"),
        mcp_binary: optional(
            env,
            "KAMN_MVP_LIVE_MCP_BINARY",
            "target/debug/kamn-mcp-server",
        ),
        mcp_endpoint: optional(env, "KAMN_MVP_LIVE_MCP_ENDPOINT", "http://127.0.0.1:18278"),
        output_root: optional(env, "KAMN_MVP_AGENT_TRANSACTION_OUTPUT_ROOT", ".kamn/demo"),
        rpc_timeout_ms: optional_positive_u64(
            env,
            "KAMN_MVP_AGENT_TRANSACTION_RPC_TIMEOUT_MS",
            180_000,
        )?,
        staging_root: staging_root(env),
        devnet_settlement_command: None,
        localhost_signed_demo_command: None,
        service_api_vertical_slice_command: None,
        service_api_websocket_command: None,
    };
    validate_config(&config)?;
    Ok(config)
}

/// Validates canonical process configuration before supervisor execution.
pub fn execute_agent_transaction_demo_contract() -> Result<String, String> {
    let env = std::env::vars().collect::<BTreeMap<_, _>>();
    let config = parse_agent_transaction_demo_config(&env)?;
    execute_agent_transaction_demo_with_config(&config)
}

/// Executes the canonical demo from an already parsed configuration.
pub fn execute_agent_transaction_demo_with_config(
    config: &AgentTransactionDemoConfig,
) -> Result<String, String> {
    super::agent_transaction_preflight::validate_agent_transaction_preflight(config)?;
    super::agent_transaction_supervisor::run_supervised_registration(config)
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

fn optional(env: &BTreeMap<String, String>, name: &str, default: &str) -> String {
    env.get(name)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or(default)
        .to_owned()
}

fn staging_root(env: &BTreeMap<String, String>) -> String {
    env.get("KAMN_MVP_AGENT_TRANSACTION_STAGING_ROOT")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            std::env::temp_dir()
                .join(format!("kamn-agent-transaction-{}", std::process::id()))
                .display()
                .to_string()
        })
}

fn positive_u64(env: &BTreeMap<String, String>, name: &str) -> Result<u64, String> {
    let value = required(env, name)?;
    value
        .parse::<u64>()
        .ok()
        .filter(|parsed| *parsed > 0)
        .ok_or_else(|| config_error(format!("{name} must be a positive integer")))
}

fn optional_positive_u64(
    env: &BTreeMap<String, String>,
    name: &str,
    default: u64,
) -> Result<u64, String> {
    match env.get(name) {
        Some(_) => positive_u64(env, name),
        None => Ok(default),
    }
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
