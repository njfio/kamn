use super::super::*;
use std::env;

use crate::drivers::mcp_agent::live_probe_tranche_two::message_query_support::{
    payload_arguments, require_non_empty, required_string_field,
};

pub(crate) fn run_live_s13_mcp_bridge_forwarding_probe() -> Result<(), String> {
    let settings = s13_settings();
    let submission = submit_bridge_message(&settings)?;
    let forwarded = forward_bridge_message(&settings, submission.bridge_id.as_str())?;
    query_bridge_message(&settings, submission.bridge_id.as_str(), &forwarded)
}

struct S13Settings {
    binary: String,
    endpoint: String,
    key_file: String,
    base_agent_name: String,
    submit_payload: String,
}

struct BridgeSubmission {
    bridge_id: String,
}

struct ForwardedBridge {
    bridge_status: String,
    target_message_id: String,
    forward_tx_hash: String,
}

fn s13_settings() -> S13Settings {
    S13Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        base_agent_name: env_var_or_default("KAMN_E2E_S13_AGENT_NAME", DEFAULT_S13_AGENT_NAME),
        submit_payload: env::var("KAMN_E2E_S13_SUBMIT_BRIDGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD.to_owned()),
    }
}

fn submit_bridge_message(settings: &S13Settings) -> Result<BridgeSubmission, String> {
    let response = run_live_s13_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-submit", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-submit-bridge-message",
        "submit_bridge_message",
        payload_arguments(settings.submit_payload.as_str()).as_str(),
    )?;
    let step = "mcp live s13 submit_bridge_message";
    let bridge_id = required_bridge_field(response.as_str(), "bridge_id", step)?;
    required_bridge_field(response.as_str(), "source_message_id", step)?;
    required_bridge_field(response.as_str(), "bridge_status", step)?;
    Ok(BridgeSubmission { bridge_id })
}

fn forward_bridge_message(
    settings: &S13Settings,
    bridge_id: &str,
) -> Result<ForwardedBridge, String> {
    let response = run_live_s13_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-forward", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-forward-bridge-message",
        "forward_bridge_message",
        bridge_id_arguments(bridge_id).as_str(),
    )?;
    let step = "mcp live s13 forward_bridge_message";
    let forwarded_bridge_id = required_bridge_field(response.as_str(), "bridge_id", step)?;
    validate_s13_bridge_id_match(bridge_id, forwarded_bridge_id.as_str(), step)?;
    Ok(ForwardedBridge {
        bridge_status: required_bridge_field(response.as_str(), "bridge_status", step)?,
        target_message_id: required_bridge_field(response.as_str(), "target_message_id", step)?,
        forward_tx_hash: required_bridge_field(response.as_str(), "forward_tx_hash", step)?,
    })
}

fn query_bridge_message(
    settings: &S13Settings,
    bridge_id: &str,
    forwarded: &ForwardedBridge,
) -> Result<(), String> {
    let response = query_bridge_response(settings, bridge_id)?;
    validate_query_bridge_response(response.as_str(), bridge_id, forwarded)
}

fn query_bridge_response(settings: &S13Settings, bridge_id: &str) -> Result<String, String> {
    let response = run_live_s13_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-query", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-query-bridge-message",
        "query_bridge_message",
        bridge_id_arguments(bridge_id).as_str(),
    )?;
    Ok(response)
}

fn validate_query_bridge_response(
    response: &str,
    bridge_id: &str,
    forwarded: &ForwardedBridge,
) -> Result<(), String> {
    let step = "mcp live s13 query_bridge_message";
    let queried_bridge_id = required_bridge_field(response, "bridge_id", step)?;
    validate_s13_bridge_id_match(bridge_id, queried_bridge_id.as_str(), step)?;
    let queried_bridge_status = required_bridge_field(response, "bridge_status", step)?;
    validate_s13_bridge_field_coherence(
        forwarded.bridge_status.as_str(),
        queried_bridge_status.as_str(),
        "bridge_status",
        step,
    )?;
    validate_forwarded_bridge_fields(response, forwarded, step)
}

fn validate_forwarded_bridge_fields(
    response: &str,
    forwarded: &ForwardedBridge,
    step: &str,
) -> Result<(), String> {
    let queried_target_message_id = required_bridge_field(response, "target_message_id", step)?;
    validate_s13_bridge_field_coherence(
        forwarded.target_message_id.as_str(),
        queried_target_message_id.as_str(),
        "target_message_id",
        step,
    )?;
    let queried_tx_hash = required_bridge_field(response, "forward_tx_hash", step)?;
    validate_s13_bridge_field_coherence(
        forwarded.forward_tx_hash.as_str(),
        queried_tx_hash.as_str(),
        "forward_tx_hash",
        step,
    )
}

fn bridge_id_arguments(bridge_id: &str) -> String {
    format!("{{\"bridge_id\":\"{}\"}}", escape_json_scalar(bridge_id))
}

fn required_bridge_field(response: &str, key: &str, step: &str) -> Result<String, String> {
    let value = required_string_field(response, key, step)?;
    require_non_empty(value.as_str(), step, key)?;
    Ok(value)
}
