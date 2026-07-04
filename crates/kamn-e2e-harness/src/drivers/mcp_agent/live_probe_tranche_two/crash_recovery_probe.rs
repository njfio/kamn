use super::super::*;
use std::env;

use super::message_query_support::{
    query_message_with_validation, send_message_with_receipt, validate_distinct_message_ids,
};

pub(crate) fn run_live_s08_mcp_crash_recovery_probe() -> Result<(), String> {
    let settings = s08_settings();
    let pre_message_id = send_and_query(
        &settings,
        settings.pre_message_payload.as_str(),
        "pre",
        "mcp live s08 pre-boundary",
    )?;
    run_boundary_health(&settings)?;
    let post_message_id = send_and_query(
        &settings,
        settings.post_message_payload.as_str(),
        "post",
        "mcp live s08 post-boundary",
    )?;
    validate_distinct_message_ids(
        pre_message_id.as_str(),
        post_message_id.as_str(),
        "mcp live s08 post-boundary send_message",
    )
}

struct S08Settings {
    binary: String,
    endpoint: String,
    key_file: String,
    base_agent_name: String,
    pre_message_payload: String,
    post_message_payload: String,
}

fn s08_settings() -> S08Settings {
    S08Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        base_agent_name: env_var_or_default("KAMN_E2E_S08_AGENT_NAME", DEFAULT_S08_AGENT_NAME),
        pre_message_payload: env::var("KAMN_E2E_S08_PRE_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S08_PRE_MESSAGE_PAYLOAD.to_owned()),
        post_message_payload: env::var("KAMN_E2E_S08_POST_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S08_POST_MESSAGE_PAYLOAD.to_owned()),
    }
}

fn send_and_query(
    settings: &S08Settings,
    payload: &str,
    suffix: &str,
    step_prefix: &str,
) -> Result<String, String> {
    let message_id = send_message(settings, payload, suffix, step_prefix)?;
    query_message(settings, message_id.as_str(), suffix, step_prefix)?;
    Ok(message_id)
}

fn send_message(
    settings: &S08Settings,
    payload: &str,
    suffix: &str,
    step_prefix: &str,
) -> Result<String, String> {
    send_message_with_receipt(
        "mcp live s08",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-{suffix}-send", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        format!("probe-send-message-{suffix}").as_str(),
        (payload, format!("{step_prefix} send_message").as_str()),
    )
}

fn query_message(
    settings: &S08Settings,
    message_id: &str,
    suffix: &str,
    step_prefix: &str,
) -> Result<(), String> {
    query_message_with_validation(
        "mcp live s08",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-{suffix}-query", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        format!("probe-query-message-{suffix}").as_str(),
        (message_id, format!("{step_prefix} query_message").as_str()),
    )
}

fn run_boundary_health(settings: &S08Settings) -> Result<(), String> {
    run_live_s08_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-boundary", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        "probe-boundary-health",
        "health",
        "{}",
    )
    .map(|_| ())
}
