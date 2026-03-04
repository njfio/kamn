use super::{CommandKind, CommandOutput, OutputFormat, ParsedCliArgs};

const DEFAULT_ENDPOINT: &str = "http://localhost:8080";
const CLI_USAGE: &str = "kamn-cli <command> [--format json|text] [--endpoint <url>] [args]";
const CLI_HELP_FLAGS: &[&str] = &["--help", "-h", "--format", "--endpoint"];
const CLI_SUPPORTED_COMMANDS: &[&str] = &[
    "register",
    "send-message",
    "create-channel",
    "list-messages",
    "query-message",
    "query-task",
    "query-agent-profile",
    "register-content",
    "expire-content",
    "tombstone-content",
    "query-content",
    "submit-bridge-message",
    "forward-bridge-message",
    "query-bridge-message",
    "create-task",
    "accept-task",
    "complete-task",
    "fund-escrow",
    "release-escrow",
    "verify-proof",
    "health",
];

fn env_var_or_default(key: &str, default: &str) -> String {
    match std::env::var(key) {
        Ok(value) => value,
        Err(_) => default.to_owned(),
    }
}

pub(super) fn is_help_request_impl<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    args.into_iter()
        .any(|value| matches!(value.as_ref(), "--help" | "-h"))
}

pub(super) fn parse_cli_args_impl<I, S>(args: I) -> Result<ParsedCliArgs, String>
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
            other => {
                if other.starts_with('-') {
                    return Err(format!("unsupported flag: {other}"));
                }
                passthrough.push(other.to_owned());
            }
        }
    }

    Ok(ParsedCliArgs {
        command,
        output_format,
        endpoint,
        passthrough,
    })
}

pub(super) fn help_output() -> CommandOutput {
    let command_list = CLI_SUPPORTED_COMMANDS.join(", ");
    let flag_list = CLI_HELP_FLAGS.join(", ");
    let text = format!("usage={CLI_USAGE}\ncommands={command_list}\nflags={flag_list}");
    let commands_json = CLI_SUPPORTED_COMMANDS
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(",");
    let flags_json = CLI_HELP_FLAGS
        .iter()
        .map(|value| format!("\"{value}\""))
        .collect::<Vec<_>>()
        .join(",");
    let json = format!(
        "{{\"usage\":\"{CLI_USAGE}\",\"commands\":[{commands_json}],\"flags\":[{flags_json}]}}"
    );
    CommandOutput::new(json, text)
}

pub(super) fn render_help_text_impl() -> String {
    let commands = CLI_SUPPORTED_COMMANDS.join(", ");
    let flags = CLI_HELP_FLAGS.join(", ");
    format!("Usage: {CLI_USAGE}\nCommands: {commands}\nFlags: {flags}")
}
