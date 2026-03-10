use super::super::*;
use std::env;

use super::message_query_support::{
    health_status, query_message_with_validation, send_message_with_receipt,
    validate_distinct_message_ids,
};

pub(crate) fn run_live_s09_mcp_transport_failover_probe() -> Result<(), String> {
    let settings = s09_settings();
    let pre_message_id = send_and_query(
        &settings,
        settings.primary_endpoint.as_str(),
        settings.pre_message_payload.as_str(),
        "pre",
        "mcp live s09 pre-failover",
    )?;
    health_status(
        "mcp live s09",
        settings.binary.as_str(),
        settings.failover_endpoint.as_str(),
        format!("{}-boundary", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-boundary-health",
        "mcp live s09 failover boundary health",
    )?;
    let post_message_id = send_and_query(
        &settings,
        settings.failover_endpoint.as_str(),
        settings.post_message_payload.as_str(),
        "post",
        "mcp live s09 post-failover",
    )?;
    validate_distinct_message_ids(
        pre_message_id.as_str(),
        post_message_id.as_str(),
        "mcp live s09 post-failover send_message",
    )
}

struct S09Settings {
    binary: String,
    primary_endpoint: String,
    failover_endpoint: String,
    key_file: String,
    base_agent_name: String,
    pre_message_payload: String,
    post_message_payload: String,
}

fn s09_settings() -> S09Settings {
    let primary_endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let failover_endpoint = env_var_or_else("KAMN_E2E_S09_FAILOVER_ENDPOINT", || {
        primary_endpoint.clone()
    });
    S09Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        primary_endpoint,
        failover_endpoint,
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        base_agent_name: env_var_or_default("KAMN_E2E_S09_AGENT_NAME", DEFAULT_S09_AGENT_NAME),
        pre_message_payload: env::var("KAMN_E2E_S09_PRE_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S09_PRE_MESSAGE_PAYLOAD.to_owned()),
        post_message_payload: env::var("KAMN_E2E_S09_POST_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S09_POST_MESSAGE_PAYLOAD.to_owned()),
    }
}

fn send_and_query(
    settings: &S09Settings,
    endpoint: &str,
    payload: &str,
    suffix: &str,
    step_prefix: &str,
) -> Result<String, String> {
    let message_id = send_message_with_receipt(
        "mcp live s09",
        settings.binary.as_str(),
        endpoint,
        format!("{}-{suffix}-send", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        format!("probe-send-message-{suffix}").as_str(),
        payload,
        format!("{step_prefix} send_message").as_str(),
    )?;
    query_message_with_validation(
        "mcp live s09",
        settings.binary.as_str(),
        endpoint,
        format!("{}-{suffix}-query", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        format!("probe-query-message-{suffix}").as_str(),
        message_id.as_str(),
        format!("{step_prefix} query_message").as_str(),
    )?;
    Ok(message_id)
}
