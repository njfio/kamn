use super::super::*;
use std::env;

use super::message_query_support::{
    health_status, query_message_with_validation, send_message_with_receipt,
};

pub(crate) fn run_live_s10_mcp_topology_coherence_probe() -> Result<(), String> {
    let settings = s10_settings();
    let message_id = send_primary_message(&settings)?;
    query_secondary_and_tertiary(&settings, message_id.as_str())?;
    verify_topology_health(&settings)
}

struct S10Settings {
    binary: String,
    primary_endpoint: String,
    secondary_endpoint: String,
    tertiary_endpoint: String,
    key_file: String,
    base_agent_name: String,
    message_payload: String,
}

fn s10_settings() -> S10Settings {
    let primary_endpoint = env::var("KAMN_E2E_S10_PRIMARY_ENDPOINT")
        .or_else(|_| env::var("KAMN_ENDPOINT"))
        .unwrap_or_else(|_| DEFAULT_KAMN_ENDPOINT.to_owned());
    let secondary_endpoint = env_var_or_else("KAMN_E2E_S10_SECONDARY_ENDPOINT", || {
        primary_endpoint.clone()
    });
    let tertiary_endpoint = env_var_or_else("KAMN_E2E_S10_TERTIARY_ENDPOINT", || {
        secondary_endpoint.clone()
    });
    S10Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        primary_endpoint,
        secondary_endpoint,
        tertiary_endpoint,
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        base_agent_name: env_var_or_default("KAMN_E2E_S10_AGENT_NAME", DEFAULT_S10_AGENT_NAME),
        message_payload: env::var("KAMN_E2E_S10_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S10_MESSAGE_PAYLOAD.to_owned()),
    }
}

fn send_primary_message(settings: &S10Settings) -> Result<String, String> {
    send_message_with_receipt(
        "mcp live s10",
        settings.binary.as_str(),
        settings.primary_endpoint.as_str(),
        format!("{}-primary-send", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-send-message-primary",
        (
            settings.message_payload.as_str(),
            "mcp live s10 primary send_message",
        ),
    )
}

fn query_secondary_and_tertiary(settings: &S10Settings, message_id: &str) -> Result<(), String> {
    query_message_with_validation(
        "mcp live s10",
        settings.binary.as_str(),
        settings.secondary_endpoint.as_str(),
        format!("{}-secondary-query", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-query-message-secondary",
        (message_id, "mcp live s10 secondary query_message"),
    )?;
    query_message_with_validation(
        "mcp live s10",
        settings.binary.as_str(),
        settings.tertiary_endpoint.as_str(),
        format!("{}-tertiary-query", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-query-message-tertiary",
        (message_id, "mcp live s10 tertiary query_message"),
    )
}

fn verify_topology_health(settings: &S10Settings) -> Result<(), String> {
    health_status(
        "mcp live s10",
        settings.binary.as_str(),
        settings.secondary_endpoint.as_str(),
        format!("{}-secondary-boundary", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-health-secondary",
        "mcp live s10 secondary health",
    )?;
    health_status(
        "mcp live s10",
        settings.binary.as_str(),
        settings.tertiary_endpoint.as_str(),
        format!("{}-tertiary-boundary", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-health-tertiary",
        "mcp live s10 tertiary health",
    )?;
    Ok(())
}
