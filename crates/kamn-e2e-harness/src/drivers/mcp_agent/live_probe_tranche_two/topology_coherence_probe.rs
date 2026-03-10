use super::super::*;
use std::env;

pub(crate) fn run_live_s10_mcp_topology_coherence_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let primary_endpoint = env::var("KAMN_E2E_S10_PRIMARY_ENDPOINT")
        .or_else(|_| env::var("KAMN_ENDPOINT"))
        .unwrap_or_else(|_| DEFAULT_KAMN_ENDPOINT.to_owned());
    let secondary_endpoint = env_var_or_else("KAMN_E2E_S10_SECONDARY_ENDPOINT", || {
        primary_endpoint.clone()
    });
    let tertiary_endpoint = env_var_or_else("KAMN_E2E_S10_TERTIARY_ENDPOINT", || {
        secondary_endpoint.clone()
    });
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S10_AGENT_NAME", DEFAULT_S10_AGENT_NAME);
    let message_payload = env::var("KAMN_E2E_S10_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S10_MESSAGE_PAYLOAD.to_owned());

    let send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let primary_send_response = run_live_s10_mcp_tool_call(
        binary.as_str(),
        primary_endpoint.as_str(),
        format!("{base_agent_name}-primary-send").as_str(),
        key_file.as_str(),
        "probe-send-message-primary",
        "send_message",
        send_arguments.as_str(),
    )?;
    let message_id = validate_s08_mcp_message_receipt_fields(
        primary_send_response.as_str(),
        "mcp live s10 primary send_message",
    )?;

    let query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(message_id.as_str())
    );
    let secondary_query_response = run_live_s10_mcp_tool_call(
        binary.as_str(),
        secondary_endpoint.as_str(),
        format!("{base_agent_name}-secondary-query").as_str(),
        key_file.as_str(),
        "probe-query-message-secondary",
        "query_message",
        query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        secondary_query_response.as_str(),
        message_id.as_str(),
        "mcp live s10 secondary query_message",
    )?;

    let tertiary_query_response = run_live_s10_mcp_tool_call(
        binary.as_str(),
        tertiary_endpoint.as_str(),
        format!("{base_agent_name}-tertiary-query").as_str(),
        key_file.as_str(),
        "probe-query-message-tertiary",
        "query_message",
        query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        tertiary_query_response.as_str(),
        message_id.as_str(),
        "mcp live s10 tertiary query_message",
    )?;

    let secondary_health_response = run_live_s10_mcp_tool_call(
        binary.as_str(),
        secondary_endpoint.as_str(),
        format!("{base_agent_name}-secondary-boundary").as_str(),
        key_file.as_str(),
        "probe-health-secondary",
        "health",
        "{}",
    )?;
    let secondary_health_status =
        json_optional_string_field(secondary_health_response.as_str(), "status").ok_or_else(
            || {
                format!(
                    "mcp live s10 secondary health response missing status field: {secondary_health_response}"
                )
            },
        )?;
    if secondary_health_status.trim().is_empty() {
        return Err("mcp live s10 secondary health check returned empty status".to_owned());
    }

    let tertiary_health_response = run_live_s10_mcp_tool_call(
        binary.as_str(),
        tertiary_endpoint.as_str(),
        format!("{base_agent_name}-tertiary-boundary").as_str(),
        key_file.as_str(),
        "probe-health-tertiary",
        "health",
        "{}",
    )?;
    let tertiary_health_status =
        json_optional_string_field(tertiary_health_response.as_str(), "status").ok_or_else(
            || {
                format!(
                    "mcp live s10 tertiary health response missing status field: {tertiary_health_response}"
                )
            },
        )?;
    if tertiary_health_status.trim().is_empty() {
        return Err("mcp live s10 tertiary health check returned empty status".to_owned());
    }

    Ok(())
}
