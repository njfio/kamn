/// Parsed `demo-mvp` command configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MvpDemoCommandConfig {
    /// Demo output root directory.
    pub output_root: String,
    /// Devnet execution mode (`optional` or `required`).
    pub devnet_mode: String,
    /// Optional Solana RPC URL for devnet-backed proof attempts.
    pub solana_rpc_url: Option<String>,
    /// Optional devnet settlement command override for tests.
    pub devnet_settlement_command: Option<Vec<String>>,
    /// Optional localhost signed demo command override for tests.
    pub localhost_signed_demo_command: Option<Vec<String>>,
    /// Optional service API vertical slice command override for tests.
    pub service_api_vertical_slice_command: Option<Vec<String>>,
    /// Optional service API websocket command override for tests.
    pub service_api_websocket_command: Option<Vec<String>>,
    /// Optional MCP-agent harness evidence artifact path.
    pub agent_harness_evidence_path: Option<String>,
}

/// Parsed `verify-mvp-demo` command configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyMvpDemoCommandConfig {
    /// Proof report JSON path.
    pub report: String,
}
