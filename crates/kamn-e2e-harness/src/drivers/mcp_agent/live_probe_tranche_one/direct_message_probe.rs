use super::super::*;
use std::env;

use crate::drivers::mcp_agent::live_probe_tranche_two::message_query_support::{
    query_message_with_validation, send_message_with_receipt,
};

pub(crate) fn run_live_s02_mcp_direct_message_probe() -> Result<(), String> {
    let settings = s02_settings();
    send_and_query(
        &settings,
        "send",
        "query",
        settings.message_payload.as_str(),
    )?;
    send_and_query(
        &settings,
        "reply",
        "query-reply",
        settings.reply_payload.as_str(),
    )?;
    Ok(())
}

struct S02Settings {
    binary: String,
    endpoint: String,
    key_file: String,
    base_agent_name: String,
    message_payload: String,
    reply_payload: String,
}

fn s02_settings() -> S02Settings {
    S02Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        base_agent_name: env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME),
        message_payload: env::var("KAMN_E2E_S02_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S02_MESSAGE_PAYLOAD.to_owned()),
        reply_payload: env::var("KAMN_E2E_S02_REPLY_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S02_REPLY_PAYLOAD.to_owned()),
    }
}

fn send_and_query(
    settings: &S02Settings,
    send_suffix: &str,
    query_suffix: &str,
    payload: &str,
) -> Result<(), String> {
    let send_step = format!("mcp live s02 {send_suffix} send_message");
    let message_id = send_message_with_receipt(
        "mcp live s02",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-s02-{send_suffix}", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        &format!("probe-send-message-{send_suffix}"),
        payload,
        send_step.as_str(),
    )?;
    let query_step = format!("mcp live s02 {query_suffix} query_message");
    query_message_with_validation(
        "mcp live s02",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-s02-{query_suffix}", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        &format!("probe-query-message-{query_suffix}"),
        message_id.as_str(),
        query_step.as_str(),
    )
}
