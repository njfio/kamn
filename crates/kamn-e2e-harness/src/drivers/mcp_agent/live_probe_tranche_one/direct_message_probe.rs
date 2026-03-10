use super::super::*;
use std::env;

pub(crate) fn run_live_s02_mcp_direct_message_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let message_payload = env::var("KAMN_E2E_S02_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S02_MESSAGE_PAYLOAD.to_owned());
    let reply_payload = env::var("KAMN_E2E_S02_REPLY_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S02_REPLY_PAYLOAD.to_owned());
    let send_agent_name = format!("{agent_name}-s02-send");
    let query_agent_name = format!("{agent_name}-s02-query");
    let reply_agent_name = format!("{agent_name}-s02-reply");
    let reply_query_agent_name = format!("{agent_name}-s02-query-reply");

    let send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let send_response = run_live_s02_mcp_tool_call(
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
            format!("mcp live s02 send_message response missing message_id field: {send_response}")
        })?;
    if message_id.trim().is_empty() {
        return Err("mcp live s02 send_message returned empty message_id".to_owned());
    }
    let send_status =
        json_optional_string_field(send_response.as_str(), "status").ok_or_else(|| {
            format!("mcp live s02 send_message response missing status field: {send_response}")
        })?;
    if send_status.trim().is_empty() {
        return Err("mcp live s02 send_message returned empty status".to_owned());
    }

    let query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(message_id.as_str())
    );
    let query_response = run_live_s02_mcp_tool_call(
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
                "mcp live s02 query_message response missing message_id field: {query_response}"
            )
        })?;
    if queried_message_id != message_id {
        return Err(format!(
            "mcp live s02 query_message returned mismatched message_id: expected={message_id}, got={queried_message_id}"
        ));
    }
    let queried_status =
        json_optional_string_field(query_response.as_str(), "status").ok_or_else(|| {
            format!("mcp live s02 query_message response missing status field: {query_response}")
        })?;
    if queried_status.trim().is_empty() {
        return Err("mcp live s02 query_message returned empty status".to_owned());
    }

    let reply_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(reply_payload.as_str())
    );
    let reply_response = run_live_s02_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        reply_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-reply",
        "send_message",
        reply_arguments.as_str(),
    )?;
    let reply_message_id =
        json_optional_string_field(reply_response.as_str(), "message_id").ok_or_else(|| {
            format!(
                "mcp live s02 reply send_message response missing message_id field: {reply_response}"
            )
        })?;
    if reply_message_id.trim().is_empty() {
        return Err("mcp live s02 reply send_message returned empty message_id".to_owned());
    }
    let reply_send_status = json_optional_string_field(reply_response.as_str(), "status")
        .ok_or_else(|| {
            format!(
                "mcp live s02 reply send_message response missing status field: {reply_response}"
            )
        })?;
    if reply_send_status.trim().is_empty() {
        return Err("mcp live s02 reply send_message returned empty status".to_owned());
    }

    let reply_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(reply_message_id.as_str())
    );
    let reply_query_response = run_live_s02_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        reply_query_agent_name.as_str(),
        key_file.as_str(),
        "probe-query-reply-message",
        "query_message",
        reply_query_arguments.as_str(),
    )?;
    let reply_queried_message_id = json_optional_string_field(
        reply_query_response.as_str(),
        "message_id",
    )
    .ok_or_else(|| {
        format!(
            "mcp live s02 reply query_message response missing message_id field: {reply_query_response}"
        )
    })?;
    if reply_queried_message_id != reply_message_id {
        return Err(format!(
            "mcp live s02 reply query_message returned mismatched message_id: expected={reply_message_id}, got={reply_queried_message_id}"
        ));
    }
    let reply_queried_status =
        json_optional_string_field(reply_query_response.as_str(), "status").ok_or_else(|| {
            format!(
                "mcp live s02 reply query_message response missing status field: {reply_query_response}"
            )
        })?;
    if reply_queried_status.trim().is_empty() {
        return Err("mcp live s02 reply query_message returned empty status".to_owned());
    }

    Ok(())
}
