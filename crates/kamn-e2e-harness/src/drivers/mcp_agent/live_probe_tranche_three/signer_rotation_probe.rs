use super::super::*;
use std::env;

pub(crate) fn run_live_s11_mcp_signer_rotation_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let primary_agent_name = env::var("KAMN_E2E_S11_PRIMARY_AGENT_NAME")
        .unwrap_or_else(|_| DEFAULT_S11_PRIMARY_AGENT_NAME.to_owned());
    let rotated_agent_name = env::var("KAMN_E2E_S11_ROTATED_AGENT_NAME")
        .unwrap_or_else(|_| format!("{primary_agent_name}-rotated"));
    let message_payload = env::var("KAMN_E2E_S11_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S11_MESSAGE_PAYLOAD.to_owned());
    let rotated_message_payload = env::var("KAMN_E2E_S11_ROTATED_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD.to_owned());
    let stale_message_payload = env::var("KAMN_E2E_S11_STALE_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S11_STALE_MESSAGE_PAYLOAD.to_owned());

    let primary_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let primary_send_response = run_live_s11_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        primary_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message-primary",
        "send_message",
        primary_send_arguments.as_str(),
    )?;
    let primary_message_id = validate_s08_mcp_message_receipt_fields(
        primary_send_response.as_str(),
        "mcp live s11 primary send_message",
    )?;

    let primary_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(primary_message_id.as_str())
    );
    let primary_query_response = run_live_s11_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{primary_agent_name}-query").as_str(),
        key_file.as_str(),
        "probe-query-message-primary",
        "query_message",
        primary_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        primary_query_response.as_str(),
        primary_message_id.as_str(),
        "mcp live s11 primary query_message",
    )?;

    let rotated_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(rotated_message_payload.as_str())
    );
    let rotated_send_response = run_live_s11_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        rotated_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message-rotated",
        "send_message",
        rotated_send_arguments.as_str(),
    )?;
    let rotated_message_id = validate_s08_mcp_message_receipt_fields(
        rotated_send_response.as_str(),
        "mcp live s11 rotated send_message",
    )?;
    if rotated_message_id == primary_message_id {
        return Err("mcp live s11 rotated send_message returned duplicate message_id".to_owned());
    }

    let rotated_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(rotated_message_id.as_str())
    );
    let rotated_query_response = run_live_s11_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{rotated_agent_name}-query").as_str(),
        key_file.as_str(),
        "probe-query-message-rotated",
        "query_message",
        rotated_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        rotated_query_response.as_str(),
        rotated_message_id.as_str(),
        "mcp live s11 rotated query_message",
    )?;

    let stale_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(stale_message_payload.as_str())
    );
    let stale_primary_error = run_live_s11_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        primary_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message-stale-primary",
        "send_message",
        stale_send_arguments.as_str(),
    )
    .err()
    .ok_or_else(|| "mcp live s11 stale-primary send_message unexpectedly succeeded".to_owned())?;
    validate_s07_replay_reason_marker(
        stale_primary_error.as_str(),
        "mcp live s11 stale-primary send_message",
    )?;

    Ok(())
}
