use super::super::*;
use std::env;

pub(crate) fn run_live_s09_mcp_transport_failover_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let primary_endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let failover_endpoint = env_var_or_else("KAMN_E2E_S09_FAILOVER_ENDPOINT", || {
        primary_endpoint.clone()
    });
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S09_AGENT_NAME", DEFAULT_S09_AGENT_NAME);
    let pre_message_payload = env::var("KAMN_E2E_S09_PRE_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S09_PRE_MESSAGE_PAYLOAD.to_owned());
    let post_message_payload = env::var("KAMN_E2E_S09_POST_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S09_POST_MESSAGE_PAYLOAD.to_owned());

    let pre_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(pre_message_payload.as_str())
    );
    let pre_send_response = run_live_s09_mcp_tool_call(
        binary.as_str(),
        primary_endpoint.as_str(),
        format!("{base_agent_name}-pre-send").as_str(),
        key_file.as_str(),
        "probe-send-message-pre",
        "send_message",
        pre_send_arguments.as_str(),
    )?;
    let pre_message_id = validate_s08_mcp_message_receipt_fields(
        pre_send_response.as_str(),
        "mcp live s09 pre-failover send_message",
    )?;

    let pre_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(pre_message_id.as_str())
    );
    let pre_query_response = run_live_s09_mcp_tool_call(
        binary.as_str(),
        primary_endpoint.as_str(),
        format!("{base_agent_name}-pre-query").as_str(),
        key_file.as_str(),
        "probe-query-message-pre",
        "query_message",
        pre_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        pre_query_response.as_str(),
        pre_message_id.as_str(),
        "mcp live s09 pre-failover query_message",
    )?;

    let boundary_response = run_live_s09_mcp_tool_call(
        binary.as_str(),
        failover_endpoint.as_str(),
        format!("{base_agent_name}-boundary").as_str(),
        key_file.as_str(),
        "probe-boundary-health",
        "health",
        "{}",
    )?;
    let boundary_status =
        json_optional_string_field(boundary_response.as_str(), "status").ok_or_else(|| {
            format!(
                "mcp live s09 failover boundary health response missing status field: {boundary_response}"
            )
        })?;
    if boundary_status.trim().is_empty() {
        return Err("mcp live s09 failover boundary health returned empty status".to_owned());
    }

    let post_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(post_message_payload.as_str())
    );
    let post_send_response = run_live_s09_mcp_tool_call(
        binary.as_str(),
        failover_endpoint.as_str(),
        format!("{base_agent_name}-post-send").as_str(),
        key_file.as_str(),
        "probe-send-message-post",
        "send_message",
        post_send_arguments.as_str(),
    )?;
    let post_message_id = validate_s08_mcp_message_receipt_fields(
        post_send_response.as_str(),
        "mcp live s09 post-failover send_message",
    )?;
    if post_message_id == pre_message_id {
        return Err(
            "mcp live s09 post-failover send_message returned duplicate message_id".to_owned(),
        );
    }

    let post_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(post_message_id.as_str())
    );
    let post_query_response = run_live_s09_mcp_tool_call(
        binary.as_str(),
        failover_endpoint.as_str(),
        format!("{base_agent_name}-post-query").as_str(),
        key_file.as_str(),
        "probe-query-message-post",
        "query_message",
        post_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        post_query_response.as_str(),
        post_message_id.as_str(),
        "mcp live s09 post-failover query_message",
    )?;

    Ok(())
}
