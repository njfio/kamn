use super::super::*;
use std::env;

use crate::drivers::mcp_agent::live_probe_tranche_two::message_query_support::{
    query_message_with_validation, send_message_with_receipt, validate_distinct_message_ids,
};

pub(crate) fn run_live_s11_mcp_signer_rotation_probe() -> Result<(), String> {
    let settings = s11_settings();
    let primary_message_id = send_and_query(
        &settings,
        settings.primary_agent_name.as_str(),
        settings.message_payload.as_str(),
        "primary",
    )?;
    let rotated_message_id = send_and_query(
        &settings,
        settings.rotated_agent_name.as_str(),
        settings.rotated_message_payload.as_str(),
        "rotated",
    )?;
    validate_distinct_message_ids(
        primary_message_id.as_str(),
        rotated_message_id.as_str(),
        "mcp live s11 rotated send_message",
    )?;
    reject_stale_primary(&settings)
}

struct S11Settings {
    binary: String,
    endpoint: String,
    key_file: String,
    primary_agent_name: String,
    rotated_agent_name: String,
    message_payload: String,
    rotated_message_payload: String,
    stale_message_payload: String,
}

fn s11_settings() -> S11Settings {
    let primary_agent_name = env::var("KAMN_E2E_S11_PRIMARY_AGENT_NAME")
        .unwrap_or_else(|_| DEFAULT_S11_PRIMARY_AGENT_NAME.to_owned());
    S11Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        rotated_agent_name: env::var("KAMN_E2E_S11_ROTATED_AGENT_NAME")
            .unwrap_or_else(|_| format!("{primary_agent_name}-rotated")),
        message_payload: env::var("KAMN_E2E_S11_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S11_MESSAGE_PAYLOAD.to_owned()),
        rotated_message_payload: env::var("KAMN_E2E_S11_ROTATED_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD.to_owned()),
        stale_message_payload: env::var("KAMN_E2E_S11_STALE_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S11_STALE_MESSAGE_PAYLOAD.to_owned()),
        primary_agent_name,
    }
}

fn send_and_query(
    settings: &S11Settings,
    agent_name: &str,
    payload: &str,
    label: &str,
) -> Result<String, String> {
    let message_id = send_message(settings, agent_name, payload, label)?;
    query_message(settings, agent_name, message_id.as_str(), label)?;
    Ok(message_id)
}

fn send_message(
    settings: &S11Settings,
    agent_name: &str,
    payload: &str,
    label: &str,
) -> Result<String, String> {
    send_message_with_receipt(
        "mcp live s11",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        agent_name,
        settings.key_file.as_str(),
        format!("probe-send-message-{label}").as_str(),
        (
            payload,
            format!("mcp live s11 {label} send_message").as_str(),
        ),
    )
}

fn query_message(
    settings: &S11Settings,
    agent_name: &str,
    message_id: &str,
    label: &str,
) -> Result<(), String> {
    query_message_with_validation(
        "mcp live s11",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{agent_name}-query").as_str(),
        settings.key_file.as_str(),
        format!("probe-query-message-{label}").as_str(),
        (
            message_id,
            format!("mcp live s11 {label} query_message").as_str(),
        ),
    )
}

fn reject_stale_primary(settings: &S11Settings) -> Result<(), String> {
    let replay_error = run_live_s11_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        settings.primary_agent_name.as_str(),
        settings.key_file.as_str(),
        "probe-send-message-stale-primary",
        "send_message",
        format!(
            "{{\"payload\":\"{}\"}}",
            escape_json_scalar(settings.stale_message_payload.as_str())
        )
        .as_str(),
    )
    .err()
    .ok_or_else(|| "mcp live s11 stale-primary send_message unexpectedly succeeded".to_owned())?;
    validate_s07_replay_reason_marker(
        replay_error.as_str(),
        "mcp live s11 stale-primary send_message",
    )
}
