use super::super::*;
use std::env;

use crate::drivers::mcp_agent::live_probe_tranche_two::message_query_support::{
    payload_arguments, require_non_empty, required_string_field,
};

pub(crate) fn run_live_s12_mcp_retention_deletion_probe() -> Result<(), String> {
    let settings = s12_settings();
    let content_id = register_content(&settings)?;
    expire_content(&settings, content_id.as_str())?;
    let tombstone = tombstone_content(&settings, content_id.as_str())?;
    query_content(&settings, content_id.as_str(), &tombstone)
}

struct S12Settings {
    binary: String,
    endpoint: String,
    key_file: String,
    base_agent_name: String,
    register_payload: String,
}

struct TombstoneState {
    lifecycle_state: String,
    redaction_status: String,
}

fn s12_settings() -> S12Settings {
    S12Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        base_agent_name: env_var_or_default("KAMN_E2E_S12_AGENT_NAME", DEFAULT_S12_AGENT_NAME),
        register_payload: env::var("KAMN_E2E_S12_REGISTER_CONTENT_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S12_REGISTER_CONTENT_PAYLOAD.to_owned()),
    }
}

fn register_content(settings: &S12Settings) -> Result<String, String> {
    let response = run_live_s12_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-register", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-register-content",
        "register_content",
        payload_arguments(settings.register_payload.as_str()).as_str(),
    )?;
    let step = "mcp live s12 register_content";
    let content_id = required_content_field(response.as_str(), "content_id", step)?;
    required_content_field(response.as_str(), "retention_class", step)?;
    Ok(content_id)
}

fn expire_content(settings: &S12Settings, content_id: &str) -> Result<String, String> {
    let response = run_live_s12_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-expire", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-expire-content",
        "expire_content",
        content_id_arguments(content_id).as_str(),
    )?;
    let step = "mcp live s12 expire_content";
    let expired_content_id = required_content_field(response.as_str(), "content_id", step)?;
    validate_s12_content_id_match(content_id, expired_content_id.as_str(), step)?;
    required_content_field(response.as_str(), "lifecycle_state", step)
}

fn tombstone_content(settings: &S12Settings, content_id: &str) -> Result<TombstoneState, String> {
    let response = run_live_s12_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-tombstone", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-tombstone-content",
        "tombstone_content",
        content_id_arguments(content_id).as_str(),
    )?;
    let step = "mcp live s12 tombstone_content";
    let tombstoned_content_id = required_content_field(response.as_str(), "content_id", step)?;
    validate_s12_content_id_match(content_id, tombstoned_content_id.as_str(), step)?;
    Ok(TombstoneState {
        lifecycle_state: required_content_field(response.as_str(), "lifecycle_state", step)?,
        redaction_status: required_content_field(response.as_str(), "redaction_status", step)?,
    })
}

fn query_content(
    settings: &S12Settings,
    content_id: &str,
    tombstone: &TombstoneState,
) -> Result<(), String> {
    let response = run_live_s12_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-query", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-query-content",
        "query_content",
        content_id_arguments(content_id).as_str(),
    )?;
    let step = "mcp live s12 query_content";
    let queried_content_id = required_content_field(response.as_str(), "content_id", step)?;
    validate_s12_content_id_match(content_id, queried_content_id.as_str(), step)?;
    let queried_lifecycle_state =
        required_content_field(response.as_str(), "lifecycle_state", step)?;
    validate_s12_content_field_coherence(
        tombstone.lifecycle_state.as_str(),
        queried_lifecycle_state.as_str(),
        "lifecycle_state",
        step,
    )?;
    let queried_redaction_status =
        required_content_field(response.as_str(), "redaction_status", step)?;
    validate_s12_content_field_coherence(
        tombstone.redaction_status.as_str(),
        queried_redaction_status.as_str(),
        "redaction_status",
        step,
    )
}

fn content_id_arguments(content_id: &str) -> String {
    format!("{{\"content_id\":\"{}\"}}", escape_json_scalar(content_id))
}

fn required_content_field(response: &str, key: &str, step: &str) -> Result<String, String> {
    let value = required_string_field(response, key, step)?;
    require_non_empty(value.as_str(), step, key)?;
    Ok(value)
}
