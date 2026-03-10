use super::super::*;
use std::env;

pub(crate) fn run_live_s08_mcp_crash_recovery_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S08_AGENT_NAME", DEFAULT_S08_AGENT_NAME);
    let pre_message_payload = env::var("KAMN_E2E_S08_PRE_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S08_PRE_MESSAGE_PAYLOAD.to_owned());
    let post_message_payload = env::var("KAMN_E2E_S08_POST_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S08_POST_MESSAGE_PAYLOAD.to_owned());

    let pre_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(pre_message_payload.as_str())
    );
    let pre_send_response = run_live_s08_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-pre-send").as_str(),
        key_file.as_str(),
        "probe-send-message-pre",
        "send_message",
        pre_send_arguments.as_str(),
    )?;
    let pre_message_id = validate_s08_mcp_message_receipt_fields(
        pre_send_response.as_str(),
        "mcp live s08 pre-boundary send_message",
    )?;

    let pre_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(pre_message_id.as_str())
    );
    let pre_query_response = run_live_s08_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-pre-query").as_str(),
        key_file.as_str(),
        "probe-query-message-pre",
        "query_message",
        pre_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        pre_query_response.as_str(),
        pre_message_id.as_str(),
        "mcp live s08 pre-boundary query_message",
    )?;

    run_live_s08_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-boundary").as_str(),
        key_file.as_str(),
        "probe-boundary-health",
        "health",
        "{}",
    )?;

    let post_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(post_message_payload.as_str())
    );
    let post_send_response = run_live_s08_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-post-send").as_str(),
        key_file.as_str(),
        "probe-send-message-post",
        "send_message",
        post_send_arguments.as_str(),
    )?;
    let post_message_id = validate_s08_mcp_message_receipt_fields(
        post_send_response.as_str(),
        "mcp live s08 post-boundary send_message",
    )?;
    if post_message_id == pre_message_id {
        return Err(
            "mcp live s08 post-boundary send_message returned duplicate message_id".to_owned(),
        );
    }

    let post_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(post_message_id.as_str())
    );
    let post_query_response = run_live_s08_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-post-query").as_str(),
        key_file.as_str(),
        "probe-query-message-post",
        "query_message",
        post_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        post_query_response.as_str(),
        post_message_id.as_str(),
        "mcp live s08 post-boundary query_message",
    )?;

    Ok(())
}
