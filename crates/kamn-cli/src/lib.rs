#![warn(missing_docs)]
//! CLI scaffold for KAMN agent operations.

/// Command modules.
pub mod commands;

const DEFAULT_ENDPOINT: &str = "http://localhost:8080";

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

    let mut output_format = OutputFormat::Text;
    let mut endpoint =
        std::env::var("KAMN_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_owned());
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
pub fn dispatch(parsed: &ParsedCliArgs) -> Result<String, kamn_agent_lib::AgentLibError> {
    match parsed.command {
        CommandKind::Register => commands::register::execute(parsed),
        CommandKind::SendMessage => commands::send_message::execute(parsed),
        CommandKind::CreateChannel => commands::create_channel::execute(parsed),
        CommandKind::ListMessages => commands::list_messages::execute(parsed),
        CommandKind::QueryMessage => commands::query_message::execute(parsed),
        CommandKind::CreateTask => commands::create_task::execute(parsed),
        CommandKind::AcceptTask => commands::accept_task::execute(parsed),
        CommandKind::CompleteTask => commands::complete_task::execute(parsed),
        CommandKind::FundEscrow => commands::fund_escrow::execute(parsed),
        CommandKind::ReleaseEscrow => commands::release_escrow::execute(parsed),
        CommandKind::VerifyProof => commands::verify_proof::execute(parsed),
        CommandKind::Health => commands::health::execute(parsed),
    }
}
