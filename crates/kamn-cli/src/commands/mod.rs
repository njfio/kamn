use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::{AgentLibError, KamnAgentHandle};

const DEFAULT_AGENT_NAME: &str = "kamn-cli";
const DEFAULT_KOLME_ENDPOINT: &str = "http://localhost:3000";

/// `accept-task` command module.
pub mod accept_task;
/// `complete-task` command module.
pub mod complete_task;
/// `create-channel` command module.
pub mod create_channel;
/// `create-task` command module.
pub mod create_task;
/// `fund-escrow` command module.
pub mod fund_escrow;
/// `health` command module.
pub mod health;
/// `list-messages` command module.
pub mod list_messages;
/// `query-message` command module.
pub mod query_message;
/// `register` command module.
pub mod register;
/// `release-escrow` command module.
pub mod release_escrow;
/// `send-message` command module.
pub mod send_message;
/// `verify-proof` command module.
pub mod verify_proof;

pub(crate) enum OutputValue {
    String(String),
    Raw(String),
    StringList(Vec<String>),
}

impl OutputValue {
    fn as_text(&self) -> String {
        match self {
            Self::String(value) => value.clone(),
            Self::Raw(value) => value.clone(),
            Self::StringList(values) => values.join(","),
        }
    }

    fn as_json(&self) -> String {
        match self {
            Self::String(value) => format!("\"{}\"", escape_json(value.as_str())),
            Self::Raw(value) => value.clone(),
            Self::StringList(values) => {
                let values = values
                    .iter()
                    .map(|value| format!("\"{}\"", escape_json(value.as_str())))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("[{values}]")
            }
        }
    }
}

pub(crate) fn command_output(fields: Vec<(&'static str, OutputValue)>) -> CommandOutput {
    let text = fields
        .iter()
        .map(|(key, value)| format!("{key}={}", value.as_text()))
        .collect::<Vec<_>>()
        .join(" ");
    let json_fields = fields
        .iter()
        .map(|(key, value)| format!("\"{}\":{}", escape_json(key), value.as_json()))
        .collect::<Vec<_>>()
        .join(",");
    CommandOutput::new(format!("{{{json_fields}}}"), text)
}

fn escape_json(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

pub(crate) fn connect_handle(args: &ParsedCliArgs) -> Result<KamnAgentHandle, AgentLibError> {
    let agent_name =
        std::env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_AGENT_NAME.to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    KamnAgentHandle::connect(
        args.endpoint.as_str(),
        kolme_endpoint.as_str(),
        agent_name.as_str(),
    )
}

pub(crate) fn required_arg<'a>(
    args: &'a ParsedCliArgs,
    index: usize,
    field: &'static str,
) -> Result<&'a str, AgentLibError> {
    let value = args
        .passthrough
        .get(index)
        .ok_or_else(|| AgentLibError::InvalidInput {
            field,
            reason: "missing required argument".to_owned(),
        })?;
    if value.trim().is_empty() {
        return Err(AgentLibError::InvalidInput {
            field,
            reason: "argument must not be empty".to_owned(),
        });
    }
    Ok(value.as_str())
}

#[cfg(test)]
mod tests {
    use super::{command_output, OutputValue};

    #[test]
    fn unit_command_output_json_escapes_control_characters() {
        let output = command_output(vec![(
            "payload",
            OutputValue::String("\"\\\n\r\t".to_owned()),
        )]);
        assert_eq!(output.json, "{\"payload\":\"\\\"\\\\\\n\\r\\t\"}");
    }

    #[test]
    fn unit_command_output_text_projection_keeps_list_comma_join() {
        let output = command_output(vec![(
            "messages",
            OutputValue::StringList(vec!["a".to_owned(), "b".to_owned()]),
        )]);
        assert_eq!(output.text, "messages=a,b");
    }
}
