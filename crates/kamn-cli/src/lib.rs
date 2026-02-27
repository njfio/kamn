#![warn(missing_docs)]
//! CLI scaffold for KAMN agent operations.

/// Command modules.
pub mod commands;

const DEFAULT_ENDPOINT: &str = "http://localhost:8080";
const HELP_TEXT: &str = "Usage: kamn-cli <command> [--endpoint <url>] [--format <json|text>] [args...]\n\nGlobal flags:\n  --help, -h        Show this help output\n  --endpoint <url>  KAMN service endpoint (default: http://localhost:8080)\n  --format <mode>   Output mode: json | text (default: json)\n\nCommands:\n  register\n  send-message\n  create-channel\n  list-messages\n  query-message\n  query-task\n  query-agent-profile\n  register-content\n  expire-content\n  tombstone-content\n  query-content\n  submit-bridge-message\n  forward-bridge-message\n  query-bridge-message\n  create-task\n  accept-task\n  complete-task\n  fund-escrow\n  release-escrow\n  verify-proof\n  health";

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

fn env_var_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(value) => value,
        Err(_) => default.to_owned(),
    }
}

/// Returns deterministic usage/help output for the CLI command surface.
pub fn render_help_text() -> &'static str {
    HELP_TEXT
}

/// Returns true when CLI arguments request help output.
pub fn is_help_request<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut iter = args.into_iter();
    let _ = iter.next();
    iter.any(|token| matches!(token.as_ref(), "--help" | "-h" | "help"))
}

/// Parses CLI arguments for phase-2 command surface contracts.
pub fn parse_cli_args<I, S>(args: I) -> Result<ParsedCliArgs, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|value| value.as_ref().to_owned())
        .collect::<Vec<_>>();

    if args.is_empty() {
        return Err("missing command".to_owned());
    }

    let mut index = 0;
    if !args[0].starts_with("--") {
        index = 1;
    }

    if index >= args.len() {
        return Err("missing command".to_owned());
    }

    let command = CommandKind::parse(args[index].as_str())?;
    index += 1;

    let mut output_format = OutputFormat::Json;
    let mut endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_ENDPOINT);
    let mut passthrough = Vec::new();

    while index < args.len() {
        let token = args[index].as_str();
        index += 1;
        match token {
            "--format" => {
                if index >= args.len() {
                    return Err("missing value for --format".to_owned());
                }
                output_format = OutputFormat::parse(args[index].as_str())?;
                index += 1;
            }
            "--endpoint" => {
                if index >= args.len() {
                    return Err("missing value for --endpoint".to_owned());
                }
                endpoint = args[index].clone();
                index += 1;
            }
            other => passthrough.push(other.to_owned()),
        }
    }

    Ok(ParsedCliArgs {
        command,
        output_format,
        endpoint,
        passthrough,
    })
}

/// Dispatches one parsed command to the corresponding phase-2 command module.
pub fn dispatch(parsed: &ParsedCliArgs) -> Result<CommandOutput, kamn_agent_lib::AgentLibError> {
    match parsed.command {
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

#[cfg(test)]
mod tests {
    use super::{is_help_request, parse_cli_args, render_help_text, OutputFormat};

    #[test]
    fn unit_cli_parser_honors_endpoint_flag() {
        let parsed = parse_cli_args(["kamn-cli", "health", "--endpoint", "http://localhost:8080"])
            .expect("parsed");
        assert_eq!(parsed.endpoint, "http://localhost:8080");
        assert_eq!(parsed.output_format, OutputFormat::Json);
    }

    #[test]
    fn unit_is_help_request_detects_help_tokens() {
        assert!(is_help_request(["kamn-cli", "--help"]));
        assert!(is_help_request(["kamn-cli", "-h"]));
        assert!(is_help_request(["kamn-cli", "help"]));
        assert!(is_help_request(["kamn-cli", "health", "--help"]));
        assert!(!is_help_request(["kamn-cli", "health"]));
    }

    #[test]
    fn unit_render_help_text_contains_usage_and_flags() {
        let help = render_help_text();
        for marker in ["Usage:", "--endpoint", "--format", "send-message", "health"] {
            assert!(
                help.contains(marker),
                "help output should contain marker `{marker}`: {help}"
            );
        }
    }
}
