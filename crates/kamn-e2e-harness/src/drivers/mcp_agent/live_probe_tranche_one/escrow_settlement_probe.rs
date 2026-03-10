use super::super::*;
use std::env;

pub(crate) fn run_live_s05_mcp_escrow_settlement_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let fund_payload = env::var("KAMN_E2E_S05_FUND_ESCROW_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S05_FUND_ESCROW_PAYLOAD.to_owned());
    let fund_agent_name = format!("{agent_name}-s05-fund");
    let release_agent_name = format!("{agent_name}-s05-release");

    let fund_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(fund_payload.as_str())
    );
    let fund_response = run_live_s05_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        fund_agent_name.as_str(),
        key_file.as_str(),
        "probe-fund-escrow",
        "fund_escrow",
        fund_arguments.as_str(),
    )?;
    let escrow_id =
        json_optional_string_field(fund_response.as_str(), "escrow_id").ok_or_else(|| {
            format!("mcp live s05 fund_escrow response missing escrow_id field: {fund_response}")
        })?;
    if escrow_id.trim().is_empty() {
        return Err("mcp live s05 fund_escrow returned empty escrow_id".to_owned());
    }
    let fund_state =
        json_optional_string_field(fund_response.as_str(), "state").ok_or_else(|| {
            format!("mcp live s05 fund_escrow response missing state field: {fund_response}")
        })?;
    if fund_state.trim().is_empty() {
        return Err("mcp live s05 fund_escrow returned empty state".to_owned());
    }

    let release_arguments = format!(
        "{{\"escrow_id\":\"{}\"}}",
        escape_json_scalar(escrow_id.as_str())
    );
    let release_response = run_live_s05_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        release_agent_name.as_str(),
        key_file.as_str(),
        "probe-release-escrow",
        "release_escrow",
        release_arguments.as_str(),
    )?;
    let released_escrow_id = json_optional_string_field(release_response.as_str(), "escrow_id")
        .ok_or_else(|| {
            format!(
                "mcp live s05 release_escrow response missing escrow_id field: {release_response}"
            )
        })?;
    let release_state =
        json_optional_string_field(release_response.as_str(), "state").ok_or_else(|| {
            format!("mcp live s05 release_escrow response missing state field: {release_response}")
        })?;
    validate_live_s05_release_escrow_response(
        escrow_id.as_str(),
        released_escrow_id.as_str(),
        release_state.as_str(),
        "mcp live s05 release_escrow",
    )?;

    Ok(())
}
