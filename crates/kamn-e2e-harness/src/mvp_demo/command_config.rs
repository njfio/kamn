/// Paths to one independent Pi A/B/C live task evidence set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTaskEvidencePaths {
    /// Minimal task handoff artifact.
    pub handoff: String,
    /// Agent A accepted-state receipt.
    pub agent_a_receipt: String,
    /// Agent B accepted-state receipt.
    pub agent_b_receipt: String,
    /// Agent C restricted-public observation.
    pub agent_c_observation: String,
}

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
    /// Optional all-or-none independent live task evidence paths.
    pub live_task_evidence: Option<LiveTaskEvidencePaths>,
    /// Optional complete Agent A, Agent B, and Agent C runtime evidence set.
    pub pi_transaction_actor_paths: Option<[String; 3]>,
}

/// Parsed `verify-mvp-demo` command configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyMvpDemoCommandConfig {
    /// Proof report JSON path.
    pub report: String,
    /// Optional agent-harness evidence validated directly against the report.
    pub agent_harness_evidence_path: Option<String>,
    /// Optional complete Agent A, Agent B, and Agent C transaction evidence set.
    pub pi_transaction_actor_paths: Option<[String; 3]>,
}
