use super::super::*;
use std::env;

use crate::drivers::mcp_agent::live_probe_tranche_two::message_query_support::{
    payload_arguments, require_non_empty, required_string_field,
};

pub(crate) fn run_live_s05_mcp_escrow_settlement_probe() -> Result<(), String> {
    let settings = s05_settings();
    let escrow_id = fund_escrow(&settings)?;
    release_escrow(&settings, escrow_id.as_str())
}

struct S05Settings {
    binary: String,
    endpoint: String,
    key_file: String,
    fund_agent_name: String,
    release_agent_name: String,
    fund_arguments: String,
}

fn s05_settings() -> S05Settings {
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let fund_payload = env::var("KAMN_E2E_S05_FUND_ESCROW_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S05_FUND_ESCROW_PAYLOAD.to_owned());
    S05Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        fund_agent_name: format!("{agent_name}-s05-fund"),
        release_agent_name: format!("{agent_name}-s05-release"),
        fund_arguments: payload_arguments(fund_payload.as_str()),
    }
}

fn fund_escrow(settings: &S05Settings) -> Result<String, String> {
    let response = run_live_s05_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        settings.fund_agent_name.as_str(),
        settings.key_file.as_str(),
        "probe-fund-escrow",
        "fund_escrow",
        settings.fund_arguments.as_str(),
    )?;
    let escrow_id =
        required_string_field(response.as_str(), "escrow_id", "mcp live s05 fund_escrow")?;
    require_non_empty(escrow_id.as_str(), "mcp live s05 fund_escrow", "escrow_id")?;
    let state = required_string_field(response.as_str(), "state", "mcp live s05 fund_escrow")?;
    require_non_empty(state.as_str(), "mcp live s05 fund_escrow", "state")?;
    Ok(escrow_id)
}

fn release_escrow(settings: &S05Settings, escrow_id: &str) -> Result<(), String> {
    let response = run_live_s05_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        settings.release_agent_name.as_str(),
        settings.key_file.as_str(),
        "probe-release-escrow",
        "release_escrow",
        format!("{{\"escrow_id\":\"{}\"}}", escape_json_scalar(escrow_id)).as_str(),
    )?;
    let released = required_string_field(
        response.as_str(),
        "escrow_id",
        "mcp live s05 release_escrow",
    )?;
    let state = required_string_field(response.as_str(), "state", "mcp live s05 release_escrow")?;
    validate_live_s05_release_escrow_response(
        escrow_id,
        released.as_str(),
        state.as_str(),
        "mcp live s05 release_escrow",
    )
}
