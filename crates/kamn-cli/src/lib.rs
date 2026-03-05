#![warn(missing_docs)]
//! CLI scaffold for KAMN agent operations.

/// Command modules.
pub mod commands;

#[path = "cli_args.rs"]
mod cli_args;
#[path = "cli_dispatch.rs"]
mod cli_dispatch;

use cli_args::{is_help_request_impl, parse_cli_args_impl, render_help_text_impl};
use cli_dispatch::dispatch_impl;

/// Output format for CLI responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// Render JSON output.
    Json,
    /// Render plain-text output.
    Text,
}

impl OutputFormat {
    fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "json" => Ok(Self::Json),
            "text" => Ok(Self::Text),
            _ => Err(format!("unsupported format: {raw}")),
        }
    }
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

impl CommandKind {
    fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "help" | "--help" | "-h" => Ok(Self::Help),
            "register" => Ok(Self::Register),
            "send-message" => Ok(Self::SendMessage),
            "create-channel" => Ok(Self::CreateChannel),
            "list-messages" => Ok(Self::ListMessages),
            "query-message" => Ok(Self::QueryMessage),
            "query-task" => Ok(Self::QueryTask),
            "query-agent-profile" => Ok(Self::QueryAgentProfile),
            "register-content" => Ok(Self::RegisterContent),
            "expire-content" => Ok(Self::ExpireContent),
            "tombstone-content" => Ok(Self::TombstoneContent),
            "query-content" => Ok(Self::QueryContent),
            "submit-bridge-message" => Ok(Self::SubmitBridgeMessage),
            "forward-bridge-message" => Ok(Self::ForwardBridgeMessage),
            "query-bridge-message" => Ok(Self::QueryBridgeMessage),
            "create-task" => Ok(Self::CreateTask),
            "accept-task" => Ok(Self::AcceptTask),
            "complete-task" => Ok(Self::CompleteTask),
            "fund-escrow" => Ok(Self::FundEscrow),
            "release-escrow" => Ok(Self::ReleaseEscrow),
            "verify-proof" => Ok(Self::VerifyProof),
            "health" => Ok(Self::Health),
            _ => Err(format!("unsupported command: {raw}")),
        }
    }
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

/// Returns whether CLI arguments include the help flag.
pub fn is_help_request<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    is_help_request_impl(args)
}

/// Parses CLI arguments for phase-2 command surface contracts.
pub fn parse_cli_args<I, S>(args: I) -> Result<ParsedCliArgs, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    parse_cli_args_impl(args)
}

/// Dispatches one parsed command to the corresponding phase-2 command module.
pub fn dispatch(parsed: &ParsedCliArgs) -> Result<CommandOutput, kamn_agent_lib::AgentLibError> {
    dispatch_impl(parsed)
}

/// Renders deterministic help text for CLI usage output.
pub fn render_help_text() -> String {
    render_help_text_impl()
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod lib_tests;
