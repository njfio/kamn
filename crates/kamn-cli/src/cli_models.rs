/// Output format for CLI responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Render JSON output.
    Json,
    /// Render plain-text output.
    Text,
}

/// Supported phase-2 CLI command kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandKind {
    /// Show CLI usage and command surface.
    Help,
    /// Register one agent identity.
    Register,
    /// Send one message.
    SendMessage,
    /// Create one channel.
    CreateChannel,
    /// List channel messages.
    ListMessages,
    /// Query one message.
    QueryMessage,
    /// Query one task.
    QueryTask,
    /// Query one agent profile.
    QueryAgentProfile,
    /// Register one content lifecycle record.
    RegisterContent,
    /// Expire one content lifecycle record.
    ExpireContent,
    /// Tombstone one content lifecycle record.
    TombstoneContent,
    /// Query one content lifecycle record.
    QueryContent,
    /// Submit one bridge message.
    SubmitBridgeMessage,
    /// Forward one submitted bridge message.
    ForwardBridgeMessage,
    /// Query one bridge message.
    QueryBridgeMessage,
    /// Create one task.
    CreateTask,
    /// Accept one task.
    AcceptTask,
    /// Complete one task.
    CompleteTask,
    /// Fund one escrow.
    FundEscrow,
    /// Release one escrow.
    ReleaseEscrow,
    /// Verify one proof.
    VerifyProof,
    /// Query health status.
    Health,
}

/// Parsed CLI arguments used by command dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCliArgs {
    /// Parsed command kind.
    pub command: CommandKind,
    /// Selected output format.
    pub output_format: OutputFormat,
    /// Service endpoint.
    pub endpoint: String,
    /// Additional command arguments not parsed by the phase-2 scaffold.
    pub passthrough: Vec<String>,
}

/// Deterministic command output projections for JSON and text modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// Structured JSON projection for machine consumption.
    pub json: String,
    /// Human-readable key/value projection.
    pub text: String,
}

impl CommandOutput {
    /// Creates one command output projection.
    pub fn new(json: String, text: String) -> Self {
        Self { json, text }
    }
}
