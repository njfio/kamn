use super::super::*;
use std::env;

pub(crate) fn run_live_s12_mcp_retention_deletion_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S12_AGENT_NAME", DEFAULT_S12_AGENT_NAME);
    let register_payload = env::var("KAMN_E2E_S12_REGISTER_CONTENT_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S12_REGISTER_CONTENT_PAYLOAD.to_owned());

    let register_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(register_payload.as_str())
    );
    let register_response = run_live_s12_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-register").as_str(),
        key_file.as_str(),
        "probe-register-content",
        "register_content",
        register_arguments.as_str(),
    )?;
    let content_id =
        json_optional_string_field(register_response.as_str(), "content_id").ok_or_else(|| {
            format!(
                "mcp live s12 register_content response missing content_id field: {register_response}"
            )
        })?;
    if content_id.trim().is_empty() {
        return Err("mcp live s12 register_content returned empty content_id".to_owned());
    }
    let retention_class =
        json_optional_string_field(register_response.as_str(), "retention_class").ok_or_else(
            || {
                format!(
                    "mcp live s12 register_content response missing retention_class field: {register_response}"
                )
            },
        )?;
    if retention_class.trim().is_empty() {
        return Err("mcp live s12 register_content returned empty retention_class".to_owned());
    }

    let expire_arguments = format!(
        "{{\"content_id\":\"{}\"}}",
        escape_json_scalar(content_id.as_str())
    );
    let expire_response = run_live_s12_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-expire").as_str(),
        key_file.as_str(),
        "probe-expire-content",
        "expire_content",
        expire_arguments.as_str(),
    )?;
    let expired_content_id = json_optional_string_field(expire_response.as_str(), "content_id")
        .ok_or_else(|| {
            format!(
                "mcp live s12 expire_content response missing content_id field: {expire_response}"
            )
        })?;
    validate_s12_content_id_match(
        content_id.as_str(),
        expired_content_id.as_str(),
        "mcp live s12 expire_content",
    )?;
    let expired_lifecycle_state =
        json_optional_string_field(expire_response.as_str(), "lifecycle_state").ok_or_else(
            || {
                format!(
            "mcp live s12 expire_content response missing lifecycle_state field: {expire_response}"
        )
            },
        )?;
    if expired_lifecycle_state.trim().is_empty() {
        return Err("mcp live s12 expire_content returned empty lifecycle_state".to_owned());
    }

    let tombstone_arguments = format!(
        "{{\"content_id\":\"{}\"}}",
        escape_json_scalar(content_id.as_str())
    );
    let tombstone_response = run_live_s12_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-tombstone").as_str(),
        key_file.as_str(),
        "probe-tombstone-content",
        "tombstone_content",
        tombstone_arguments.as_str(),
    )?;
    let tombstoned_content_id =
        json_optional_string_field(tombstone_response.as_str(), "content_id").ok_or_else(|| {
            format!(
                "mcp live s12 tombstone_content response missing content_id field: {tombstone_response}"
            )
        })?;
    validate_s12_content_id_match(
        content_id.as_str(),
        tombstoned_content_id.as_str(),
        "mcp live s12 tombstone_content",
    )?;
    let tombstoned_lifecycle_state =
        json_optional_string_field(tombstone_response.as_str(), "lifecycle_state").ok_or_else(
            || {
                format!(
                    "mcp live s12 tombstone_content response missing lifecycle_state field: {tombstone_response}"
                )
            },
        )?;
    if tombstoned_lifecycle_state.trim().is_empty() {
        return Err("mcp live s12 tombstone_content returned empty lifecycle_state".to_owned());
    }
    let tombstoned_redaction_status =
        json_optional_string_field(tombstone_response.as_str(), "redaction_status").ok_or_else(
            || {
                format!(
                    "mcp live s12 tombstone_content response missing redaction_status field: {tombstone_response}"
                )
            },
        )?;
    if tombstoned_redaction_status.trim().is_empty() {
        return Err("mcp live s12 tombstone_content returned empty redaction_status".to_owned());
    }

    let query_arguments = format!(
        "{{\"content_id\":\"{}\"}}",
        escape_json_scalar(content_id.as_str())
    );
    let query_response = run_live_s12_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-query").as_str(),
        key_file.as_str(),
        "probe-query-content",
        "query_content",
        query_arguments.as_str(),
    )?;
    let queried_content_id = json_optional_string_field(query_response.as_str(), "content_id")
        .ok_or_else(|| {
            format!(
                "mcp live s12 query_content response missing content_id field: {query_response}"
            )
        })?;
    validate_s12_content_id_match(
        content_id.as_str(),
        queried_content_id.as_str(),
        "mcp live s12 query_content",
    )?;
    let queried_lifecycle_state =
        json_optional_string_field(query_response.as_str(), "lifecycle_state").ok_or_else(
            || {
                format!(
            "mcp live s12 query_content response missing lifecycle_state field: {query_response}"
        )
            },
        )?;
    validate_s12_content_field_coherence(
        tombstoned_lifecycle_state.as_str(),
        queried_lifecycle_state.as_str(),
        "lifecycle_state",
        "mcp live s12 query_content",
    )?;
    let queried_redaction_status =
        json_optional_string_field(query_response.as_str(), "redaction_status").ok_or_else(
            || {
                format!(
            "mcp live s12 query_content response missing redaction_status field: {query_response}"
        )
            },
        )?;
    validate_s12_content_field_coherence(
        tombstoned_redaction_status.as_str(),
        queried_redaction_status.as_str(),
        "redaction_status",
        "mcp live s12 query_content",
    )?;

    Ok(())
}
