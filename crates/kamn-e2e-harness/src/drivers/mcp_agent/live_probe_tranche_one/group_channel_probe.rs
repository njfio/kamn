use super::super::*;
use std::env;

pub(crate) fn run_live_s03_mcp_group_channel_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let channel_payload = env::var("KAMN_E2E_S03_CHANNEL_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S03_CHANNEL_PAYLOAD.to_owned());
    let message_payload = env::var("KAMN_E2E_S03_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S03_MESSAGE_PAYLOAD.to_owned());
    let create_agent_name = format!("{agent_name}-s03-create-channel");
    let send_agent_name = format!("{agent_name}-s03-send-message");
    let query_agent_name = format!("{agent_name}-s03-query-message");
    let list_agent_name = format!("{agent_name}-s03-list-messages");

    let create_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(channel_payload.as_str())
    );
    let create_response = run_live_s03_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        create_agent_name.as_str(),
        key_file.as_str(),
        "probe-create-channel",
        "create_channel",
        create_arguments.as_str(),
    )?;
    let channel_id = json_optional_string_field(create_response.as_str(), "channel_id")
        .ok_or_else(|| {
            format!(
                "mcp live s03 create_channel response missing channel_id field: {create_response}"
            )
        })?;
    if channel_id.trim().is_empty() {
        return Err("mcp live s03 create_channel returned empty channel_id".to_owned());
    }
    let create_status =
        json_optional_string_field(create_response.as_str(), "status").ok_or_else(|| {
            format!("mcp live s03 create_channel response missing status field: {create_response}")
        })?;
    if create_status.trim().is_empty() {
        return Err("mcp live s03 create_channel returned empty status".to_owned());
    }

    let send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let send_response = run_live_s03_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        send_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message",
        "send_message",
        send_arguments.as_str(),
    )?;
    let message_id =
        json_optional_string_field(send_response.as_str(), "message_id").ok_or_else(|| {
            format!("mcp live s03 send_message response missing message_id field: {send_response}")
        })?;
    if message_id.trim().is_empty() {
        return Err("mcp live s03 send_message returned empty message_id".to_owned());
    }
    let send_status =
        json_optional_string_field(send_response.as_str(), "status").ok_or_else(|| {
            format!("mcp live s03 send_message response missing status field: {send_response}")
        })?;
    if send_status.trim().is_empty() {
        return Err("mcp live s03 send_message returned empty status".to_owned());
    }

    let query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(message_id.as_str())
    );
    let query_response = run_live_s03_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        query_agent_name.as_str(),
        key_file.as_str(),
        "probe-query-message",
        "query_message",
        query_arguments.as_str(),
    )?;
    let queried_message_id = json_optional_string_field(query_response.as_str(), "message_id")
        .ok_or_else(|| {
            format!(
                "mcp live s03 query_message response missing message_id field: {query_response}"
            )
        })?;
    if queried_message_id != message_id {
        return Err(format!(
            "mcp live s03 query_message returned mismatched message_id: expected={message_id}, got={queried_message_id}"
        ));
    }
    let queried_status =
        json_optional_string_field(query_response.as_str(), "status").ok_or_else(|| {
            format!("mcp live s03 query_message response missing status field: {query_response}")
        })?;
    if queried_status.trim().is_empty() {
        return Err("mcp live s03 query_message returned empty status".to_owned());
    }

    let list_arguments = format!(
        "{{\"channel_id\":\"{}\"}}",
        escape_json_scalar(channel_id.as_str())
    );
    let list_response = run_live_s03_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        list_agent_name.as_str(),
        key_file.as_str(),
        "probe-list-messages",
        "list_messages",
        list_arguments.as_str(),
    )?;
    let listed_channel_id = json_optional_string_field(list_response.as_str(), "channel_id")
        .ok_or_else(|| {
            format!("mcp live s03 list_messages response missing channel_id field: {list_response}")
        })?;
    if listed_channel_id != channel_id {
        return Err(format!(
            "mcp live s03 list_messages returned mismatched channel_id: expected={channel_id}, got={listed_channel_id}"
        ));
    }
    if !list_response.contains(r#""messages":["#) {
        return Err(format!(
            "mcp live s03 list_messages response missing messages field: {list_response}"
        ));
    }

    Ok(())
}
