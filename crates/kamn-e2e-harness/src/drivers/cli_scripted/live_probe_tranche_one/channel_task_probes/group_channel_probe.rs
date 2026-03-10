use super::super::super::{
    parse_text_output_field, run_cli_command_capture_stdout_with_agent_name,
    DEFAULT_S03_AGENT_NAME, DEFAULT_S03_CHANNEL_PAYLOAD, DEFAULT_S03_MESSAGE_PAYLOAD,
};
use super::super::{agent_name, cli_binary, endpoint, env_payload, validate_non_empty};

pub(super) fn run_live_s03_cli_group_channel_probe() -> Result<(), String> {
    let agent_name = agent_name(DEFAULT_S03_AGENT_NAME);
    let channel_id = create_channel(agent_name.as_str())?;
    let message_id = send_message(agent_name.as_str())?;
    query_message(agent_name.as_str(), message_id.as_str())?;
    list_messages(agent_name.as_str(), channel_id.as_str())
}

fn create_channel(agent_name: &str) -> Result<String, String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "create-channel",
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            env_payload("KAMN_E2E_S03_CHANNEL_PAYLOAD", DEFAULT_S03_CHANNEL_PAYLOAD).as_str(),
        ],
        "cli live s03 create-channel",
        format!("{agent_name}-create-channel").as_str(),
    )?;
    let channel_id = require_field(output.as_str(), "channel_id", "cli live s03 create-channel")?;
    validate_non_empty(
        channel_id,
        "cli live s03 create-channel returned empty channel_id",
    )?;
    validate_non_empty(
        require_field(output.as_str(), "status", "cli live s03 create-channel")?,
        "cli live s03 create-channel returned empty status",
    )?;
    Ok(channel_id.to_owned())
}

fn send_message(agent_name: &str) -> Result<String, String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            env_payload("KAMN_E2E_S03_MESSAGE_PAYLOAD", DEFAULT_S03_MESSAGE_PAYLOAD).as_str(),
        ],
        "cli live s03 send-message",
        format!("{agent_name}-send-message").as_str(),
    )?;
    let message_id = require_field(output.as_str(), "message_id", "cli live s03 send-message")?;
    validate_non_empty(
        message_id,
        "cli live s03 send-message returned empty message_id",
    )?;
    validate_non_empty(
        require_field(output.as_str(), "status", "cli live s03 send-message")?,
        "cli live s03 send-message returned empty status",
    )?;
    Ok(message_id.to_owned())
}

fn query_message(agent_name: &str, message_id: &str) -> Result<(), String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            message_id,
        ],
        "cli live s03 query-message",
        format!("{agent_name}-query-message").as_str(),
    )?;
    validate_message_id(output.as_str(), message_id)?;
    validate_non_empty(
        require_field(output.as_str(), "status", "cli live s03 query-message")?,
        "cli live s03 query-message returned empty status",
    )
}

fn list_messages(agent_name: &str, channel_id: &str) -> Result<(), String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "list-messages",
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            channel_id,
        ],
        "cli live s03 list-messages",
        format!("{agent_name}-list-messages").as_str(),
    )?;
    let listed_channel_id =
        require_field(output.as_str(), "channel_id", "cli live s03 list-messages")?;
    if listed_channel_id != channel_id {
        return Err(format!(
            "cli live s03 list-messages returned mismatched channel_id: expected={channel_id}, got={listed_channel_id}"
        ));
    }
    let _ = require_field(output.as_str(), "messages", "cli live s03 list-messages")?;
    Ok(())
}

fn require_field<'a>(output: &'a str, key: &str, step: &str) -> Result<&'a str, String> {
    parse_text_output_field(output, key)
        .ok_or_else(|| format!("{step} response missing {key} field: {output}"))
}

fn validate_message_id(output: &str, message_id: &str) -> Result<(), String> {
    let queried_message_id = require_field(output, "message_id", "cli live s03 query-message")?;
    if queried_message_id == message_id {
        return Ok(());
    }
    Err(format!(
        "cli live s03 query-message returned mismatched message_id: expected={message_id}, got={queried_message_id}"
    ))
}
