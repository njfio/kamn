use super::super::*;
use std::env;

pub(crate) fn run_live_s13_mcp_bridge_forwarding_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S13_AGENT_NAME", DEFAULT_S13_AGENT_NAME);
    let submit_payload = env::var("KAMN_E2E_S13_SUBMIT_BRIDGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD.to_owned());

    let submit_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(submit_payload.as_str())
    );
    let submit_response = run_live_s13_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-submit").as_str(),
        key_file.as_str(),
        "probe-submit-bridge-message",
        "submit_bridge_message",
        submit_arguments.as_str(),
    )?;
    let bridge_id = json_optional_string_field(submit_response.as_str(), "bridge_id").ok_or_else(
        || {
            format!(
                "mcp live s13 submit_bridge_message response missing bridge_id field: {submit_response}"
            )
        },
    )?;
    if bridge_id.trim().is_empty() {
        return Err("mcp live s13 submit_bridge_message returned empty bridge_id".to_owned());
    }
    let source_message_id =
        json_optional_string_field(submit_response.as_str(), "source_message_id").ok_or_else(
            || {
                format!(
                    "mcp live s13 submit_bridge_message response missing source_message_id field: {submit_response}"
                )
            },
        )?;
    if source_message_id.trim().is_empty() {
        return Err(
            "mcp live s13 submit_bridge_message returned empty source_message_id".to_owned(),
        );
    }
    let submit_bridge_status =
        json_optional_string_field(submit_response.as_str(), "bridge_status").ok_or_else(|| {
            format!(
                "mcp live s13 submit_bridge_message response missing bridge_status field: {submit_response}"
            )
        })?;
    if submit_bridge_status.trim().is_empty() {
        return Err("mcp live s13 submit_bridge_message returned empty bridge_status".to_owned());
    }

    let forward_arguments = format!(
        "{{\"bridge_id\":\"{}\"}}",
        escape_json_scalar(bridge_id.as_str())
    );
    let forward_response = run_live_s13_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-forward").as_str(),
        key_file.as_str(),
        "probe-forward-bridge-message",
        "forward_bridge_message",
        forward_arguments.as_str(),
    )?;
    let forwarded_bridge_id = json_optional_string_field(forward_response.as_str(), "bridge_id")
        .ok_or_else(|| {
            format!(
                "mcp live s13 forward_bridge_message response missing bridge_id field: {forward_response}"
            )
        })?;
    validate_s13_bridge_id_match(
        bridge_id.as_str(),
        forwarded_bridge_id.as_str(),
        "mcp live s13 forward_bridge_message",
    )?;
    let forwarded_bridge_status =
        json_optional_string_field(forward_response.as_str(), "bridge_status").ok_or_else(|| {
            format!(
                "mcp live s13 forward_bridge_message response missing bridge_status field: {forward_response}"
            )
        })?;
    if forwarded_bridge_status.trim().is_empty() {
        return Err("mcp live s13 forward_bridge_message returned empty bridge_status".to_owned());
    }
    let forwarded_target_message_id = json_optional_string_field(
        forward_response.as_str(),
        "target_message_id",
    )
    .ok_or_else(|| {
        format!(
            "mcp live s13 forward_bridge_message response missing target_message_id field: {forward_response}"
        )
    })?;
    if forwarded_target_message_id.trim().is_empty() {
        return Err(
            "mcp live s13 forward_bridge_message returned empty target_message_id".to_owned(),
        );
    }
    let forwarded_tx_hash =
        json_optional_string_field(forward_response.as_str(), "forward_tx_hash").ok_or_else(
            || {
                format!(
                    "mcp live s13 forward_bridge_message response missing forward_tx_hash field: {forward_response}"
                )
            },
        )?;
    if forwarded_tx_hash.trim().is_empty() {
        return Err(
            "mcp live s13 forward_bridge_message returned empty forward_tx_hash".to_owned(),
        );
    }

    let query_arguments = format!(
        "{{\"bridge_id\":\"{}\"}}",
        escape_json_scalar(bridge_id.as_str())
    );
    let query_response = run_live_s13_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-query").as_str(),
        key_file.as_str(),
        "probe-query-bridge-message",
        "query_bridge_message",
        query_arguments.as_str(),
    )?;
    let queried_bridge_id = json_optional_string_field(query_response.as_str(), "bridge_id")
        .ok_or_else(|| {
            format!(
                "mcp live s13 query_bridge_message response missing bridge_id field: {query_response}"
            )
        })?;
    validate_s13_bridge_id_match(
        bridge_id.as_str(),
        queried_bridge_id.as_str(),
        "mcp live s13 query_bridge_message",
    )?;
    let queried_bridge_status =
        json_optional_string_field(query_response.as_str(), "bridge_status").ok_or_else(|| {
            format!(
                "mcp live s13 query_bridge_message response missing bridge_status field: {query_response}"
            )
        })?;
    validate_s13_bridge_field_coherence(
        forwarded_bridge_status.as_str(),
        queried_bridge_status.as_str(),
        "bridge_status",
        "mcp live s13 query_bridge_message",
    )?;
    let queried_target_message_id = json_optional_string_field(
        query_response.as_str(),
        "target_message_id",
    )
    .ok_or_else(|| {
        format!(
            "mcp live s13 query_bridge_message response missing target_message_id field: {query_response}"
        )
    })?;
    validate_s13_bridge_field_coherence(
        forwarded_target_message_id.as_str(),
        queried_target_message_id.as_str(),
        "target_message_id",
        "mcp live s13 query_bridge_message",
    )?;
    let queried_tx_hash =
        json_optional_string_field(query_response.as_str(), "forward_tx_hash").ok_or_else(|| {
            format!(
                "mcp live s13 query_bridge_message response missing forward_tx_hash field: {query_response}"
            )
        })?;
    validate_s13_bridge_field_coherence(
        forwarded_tx_hash.as_str(),
        queried_tx_hash.as_str(),
        "forward_tx_hash",
        "mcp live s13 query_bridge_message",
    )?;

    Ok(())
}
