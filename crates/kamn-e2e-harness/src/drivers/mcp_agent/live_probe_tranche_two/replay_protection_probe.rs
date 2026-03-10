use super::super::*;
use std::env;

pub(crate) fn run_live_s07_mcp_replay_protection_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S07_AGENT_NAME", DEFAULT_S07_AGENT_NAME);
    let message_payload = env::var("KAMN_E2E_S07_REPLAY_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S07_MESSAGE_PAYLOAD.to_owned());
    let replay_agent_name = format!(
        "{base_agent_name}-{}",
        live_s07_probe_agent_suffix().as_str()
    );

    let send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let first_response = run_live_s07_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        replay_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message-initial",
        "send_message",
        send_arguments.as_str(),
    )?;
    let message_id =
        json_optional_string_field(first_response.as_str(), "message_id").ok_or_else(|| {
            format!(
                "mcp live s07 initial send_message response missing message_id field: {first_response}"
            )
        })?;
    if message_id.trim().is_empty() {
        return Err("mcp live s07 initial send_message returned empty message_id".to_owned());
    }
    let send_status =
        json_optional_string_field(first_response.as_str(), "status").ok_or_else(|| {
            format!(
                "mcp live s07 initial send_message response missing status field: {first_response}"
            )
        })?;
    if send_status.trim().is_empty() {
        return Err("mcp live s07 initial send_message returned empty status".to_owned());
    }

    let replay_error = run_live_s07_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        replay_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message-replay",
        "send_message",
        send_arguments.as_str(),
    )
    .err()
    .ok_or_else(|| "mcp live s07 replay send_message unexpectedly succeeded".to_owned())?;
    validate_s07_replay_reason_marker(replay_error.as_str(), "mcp live s07 replay send_message")?;

    Ok(())
}
