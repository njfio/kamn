use super::super::*;
use std::env;

use crate::drivers::mcp_agent::live_probe_tranche_two::message_query_support::{
    payload_arguments, query_message_with_validation, require_non_empty, required_string_field,
    send_message_with_receipt,
};

pub(crate) fn run_live_s03_mcp_group_channel_probe() -> Result<(), String> {
    let settings = s03_settings();
    let channel_id = create_channel(&settings)?;
    let message_id = send_channel_message(&settings)?;
    query_message(&settings, message_id.as_str())?;
    list_messages(&settings, channel_id.as_str())
}

struct S03Settings {
    binary: String,
    endpoint: String,
    key_file: String,
    base_agent_name: String,
    channel_payload: String,
    message_payload: String,
}

fn s03_settings() -> S03Settings {
    S03Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        base_agent_name: env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME),
        channel_payload: env::var("KAMN_E2E_S03_CHANNEL_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S03_CHANNEL_PAYLOAD.to_owned()),
        message_payload: env::var("KAMN_E2E_S03_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S03_MESSAGE_PAYLOAD.to_owned()),
    }
}

fn create_channel(settings: &S03Settings) -> Result<String, String> {
    let response = run_live_s03_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-s03-create-channel", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-create-channel",
        "create_channel",
        payload_arguments(settings.channel_payload.as_str()).as_str(),
    )?;
    validate_create_channel(response.as_str())
}

fn validate_create_channel(response: &str) -> Result<String, String> {
    let step = "mcp live s03 create_channel";
    let channel_id = required_string_field(response, "channel_id", step)?;
    require_non_empty(channel_id.as_str(), step, "channel_id")?;
    let status = required_string_field(response, "status", step)?;
    require_non_empty(status.as_str(), step, "status")?;
    Ok(channel_id)
}

fn send_channel_message(settings: &S03Settings) -> Result<String, String> {
    send_message_with_receipt(
        "mcp live s03",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-s03-send-message", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-send-message",
        (
            settings.message_payload.as_str(),
            "mcp live s03 send_message",
        ),
    )
}

fn query_message(settings: &S03Settings, message_id: &str) -> Result<(), String> {
    query_message_with_validation(
        "mcp live s03",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-s03-query-message", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-query-message",
        (message_id, "mcp live s03 query_message"),
    )
}

fn list_messages(settings: &S03Settings, channel_id: &str) -> Result<(), String> {
    let response = list_messages_response(settings, channel_id)?;
    validate_list_messages(response.as_str(), channel_id)
}

fn list_messages_response(settings: &S03Settings, channel_id: &str) -> Result<String, String> {
    run_live_s03_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-s03-list-messages", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-list-messages",
        "list_messages",
        format!("{{\"channel_id\":\"{}\"}}", escape_json_scalar(channel_id)).as_str(),
    )
}

fn validate_list_messages(response: &str, channel_id: &str) -> Result<(), String> {
    let listed = required_string_field(response, "channel_id", "mcp live s03 list_messages")?;
    if listed != channel_id {
        return Err(format!(
            "mcp live s03 list_messages returned mismatched channel_id: expected={channel_id}, got={listed}"
        ));
    }
    if response.contains(r#"\"messages\":["#) {
        return Ok(());
    }
    Err(format!(
        "mcp live s03 list_messages response missing messages field: {response}"
    ))
}
