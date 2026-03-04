#![warn(missing_docs)]
//! CLI scaffold for KAMN agent operations.

/// Command modules.
pub mod commands;

#[path = "cli_args.rs"]
mod cli_args;

use cli_args::{help_output, is_help_request_impl, parse_cli_args_impl, render_help_text_impl};

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
    match parsed.command {
        CommandKind::Help => Ok(help_output()),
        CommandKind::Register => commands::register::execute(parsed),
        CommandKind::SendMessage => commands::send_message::execute(parsed),
        CommandKind::CreateChannel => commands::create_channel::execute(parsed),
        CommandKind::ListMessages => commands::list_messages::execute(parsed),
        CommandKind::QueryMessage => commands::query_message::execute(parsed),
        CommandKind::QueryTask => commands::query_task::execute(parsed),
        CommandKind::QueryAgentProfile => commands::query_agent_profile::execute(parsed),
        CommandKind::RegisterContent => commands::register_content::execute(parsed),
        CommandKind::ExpireContent => commands::expire_content::execute(parsed),
        CommandKind::TombstoneContent => commands::tombstone_content::execute(parsed),
        CommandKind::QueryContent => commands::query_content::execute(parsed),
        CommandKind::SubmitBridgeMessage => commands::submit_bridge_message::execute(parsed),
        CommandKind::ForwardBridgeMessage => commands::forward_bridge_message::execute(parsed),
        CommandKind::QueryBridgeMessage => commands::query_bridge_message::execute(parsed),
        CommandKind::CreateTask => commands::create_task::execute(parsed),
        CommandKind::AcceptTask => commands::accept_task::execute(parsed),
        CommandKind::CompleteTask => commands::complete_task::execute(parsed),
        CommandKind::FundEscrow => commands::fund_escrow::execute(parsed),
        CommandKind::ReleaseEscrow => commands::release_escrow::execute(parsed),
        CommandKind::VerifyProof => commands::verify_proof::execute(parsed),
        CommandKind::Health => commands::health::execute(parsed),
    }
}

/// Renders deterministic help text for CLI usage output.
pub fn render_help_text() -> String {
    render_help_text_impl()
}

#[cfg(test)]
mod tests {
    use super::{
        dispatch, is_help_request, parse_cli_args, render_help_text, CommandKind, OutputFormat,
    };

    #[test]
    fn unit_cli_parser_honors_endpoint_flag() {
        let parsed = parse_cli_args(["kamn-cli", "health", "--endpoint", "http://localhost:8080"])
            .expect("parsed");
        assert_eq!(parsed.endpoint, "http://localhost:8080");
        assert_eq!(parsed.output_format, OutputFormat::Json);
    }

    #[test]
    fn regression_issue_6198_cli_parser_accepts_help_flag_as_command() {
        let parsed = parse_cli_args(["kamn-cli", "--help"]).expect("help command should parse");
        assert_eq!(parsed.command, CommandKind::Help);
    }

    #[test]
    fn regression_issue_6198_cli_dispatch_renders_usage_surface() {
        let parsed = parse_cli_args(["kamn-cli", "--help"]).expect("help command should parse");
        let output = dispatch(&parsed).expect("help command should dispatch");
        assert!(output.text.contains("usage=kamn-cli <command>"));
        assert!(output.text.contains("commands=register"));
        assert!(output.text.contains("flags=--help"));
        assert!(output.json.contains("\"usage\":\"kamn-cli <command>"));
        assert!(output.json.contains("\"commands\":[\"register\""));
    }

    #[test]
    fn regression_issue_6213_cli_parser_rejects_unknown_long_flag() {
        let error = parse_cli_args(["kamn-cli", "health", "--unknown"]).expect_err("must fail");
        assert_eq!(error, "unsupported flag: --unknown");
    }

    #[test]
    fn regression_issue_6213_cli_parser_rejects_unknown_short_flag() {
        let error = parse_cli_args(["kamn-cli", "health", "-x"]).expect_err("must fail");
        assert_eq!(error, "unsupported flag: -x");
    }

    #[test]
    fn regression_issue_6213_cli_parser_keeps_non_flag_passthrough() {
        let parsed = parse_cli_args(["kamn-cli", "send-message", "payload.json"])
            .expect("non-flag positional arguments should pass through");
        assert_eq!(parsed.passthrough, vec!["payload.json".to_owned()]);
    }

    #[test]
    fn regression_issue_6219_help_request_true_for_long_flag() {
        assert!(is_help_request(["kamn-cli", "--help"]));
    }

    #[test]
    fn regression_issue_6219_help_request_true_for_short_flag() {
        assert!(is_help_request(["kamn-cli", "-h"]));
    }

    #[test]
    fn regression_issue_6219_help_request_false_without_help_flags() {
        assert!(!is_help_request(["kamn-cli", "health"]));
    }

    #[test]
    fn regression_issue_6219_render_help_text_includes_usage_commands_and_flags() {
        let help = render_help_text();
        assert!(help.contains("Usage: kamn-cli <command>"));
        assert!(help.contains("Commands: register"));
        assert!(help.contains("Flags: --help"));
    }
}
