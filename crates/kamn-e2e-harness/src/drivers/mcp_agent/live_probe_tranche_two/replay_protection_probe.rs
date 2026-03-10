use super::super::*;
use std::env;

use super::message_query_support::{payload_arguments, validate_s08_mcp_message_receipt_fields};

pub(crate) fn run_live_s07_mcp_replay_protection_probe() -> Result<(), String> {
    let settings = s07_settings();
    send_initial_message(&settings)?;
    reject_replay(&settings)
}

struct S07Settings {
    binary: String,
    endpoint: String,
    key_file: String,
    replay_agent_name: String,
    send_arguments: String,
}

fn s07_settings() -> S07Settings {
    let base_agent_name = env_var_or_default("KAMN_E2E_S07_AGENT_NAME", DEFAULT_S07_AGENT_NAME);
    let message_payload = env::var("KAMN_E2E_S07_REPLAY_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S07_MESSAGE_PAYLOAD.to_owned());
    S07Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        replay_agent_name: format!(
            "{base_agent_name}-{}",
            live_s07_probe_agent_suffix().as_str()
        ),
        send_arguments: payload_arguments(message_payload.as_str()),
    }
}

fn send_initial_message(settings: &S07Settings) -> Result<(), String> {
    let first_response = run_live_s07_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        settings.replay_agent_name.as_str(),
        settings.key_file.as_str(),
        "probe-send-message-initial",
        "send_message",
        settings.send_arguments.as_str(),
    )?;
    validate_s08_mcp_message_receipt_fields(
        first_response.as_str(),
        "mcp live s07 initial send_message",
    )
    .map(|_| ())
}

fn reject_replay(settings: &S07Settings) -> Result<(), String> {
    let replay_error = run_live_s07_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        settings.replay_agent_name.as_str(),
        settings.key_file.as_str(),
        "probe-send-message-replay",
        "send_message",
        settings.send_arguments.as_str(),
    )
    .err()
    .ok_or_else(|| "mcp live s07 replay send_message unexpectedly succeeded".to_owned())?;
    validate_s07_replay_reason_marker(replay_error.as_str(), "mcp live s07 replay send_message")
}
