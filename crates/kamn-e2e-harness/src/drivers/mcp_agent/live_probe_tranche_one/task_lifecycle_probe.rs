use super::super::*;
use std::env;

pub(crate) fn run_live_s04_mcp_task_lifecycle_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let create_task_payload = env::var("KAMN_E2E_S04_CREATE_TASK_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S04_CREATE_TASK_PAYLOAD.to_owned());
    let create_agent_name = format!("{agent_name}-s04-create");
    let fund_agent_name = format!("{agent_name}-s04-fund");
    let accept_agent_name = format!("{agent_name}-s04-accept");
    let complete_agent_name = format!("{agent_name}-s04-complete");
    let release_agent_name = format!("{agent_name}-s04-release");

    let create_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(create_task_payload.as_str())
    );
    let create_response = run_live_s04_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        create_agent_name.as_str(),
        key_file.as_str(),
        "probe-create-task",
        "create_task",
        create_arguments.as_str(),
    )?;
    let task_id =
        json_optional_string_field(create_response.as_str(), "task_id").ok_or_else(|| {
            format!("mcp live s04 create_task response missing task_id field: {create_response}")
        })?;
    if task_id.trim().is_empty() {
        return Err("mcp live s04 create_task returned empty task_id".to_owned());
    }

    let fund_payload = format!(
        "{{\"task_id\":\"{}\",\"amount\":{}}}",
        escape_json_scalar(task_id.as_str()),
        DEFAULT_S04_ESCROW_AMOUNT
    );
    let fund_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(fund_payload.as_str())
    );
    let fund_response = run_live_s04_mcp_tool_call(
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
            format!("mcp live s04 fund_escrow response missing escrow_id field: {fund_response}")
        })?;
    if escrow_id.trim().is_empty() {
        return Err("mcp live s04 fund_escrow returned empty escrow_id".to_owned());
    }

    let accept_arguments = format!(
        "{{\"task_id\":\"{}\"}}",
        escape_json_scalar(task_id.as_str())
    );
    let accept_response = run_live_s04_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        accept_agent_name.as_str(),
        key_file.as_str(),
        "probe-accept-task",
        "accept_task",
        accept_arguments.as_str(),
    )?;
    let accept_state =
        json_optional_string_field(accept_response.as_str(), "state").ok_or_else(|| {
            format!("mcp live s04 accept_task response missing state field: {accept_response}")
        })?;
    if accept_state.trim().is_empty() {
        return Err("mcp live s04 accept_task returned empty state".to_owned());
    }

    let complete_arguments = format!(
        "{{\"task_id\":\"{}\"}}",
        escape_json_scalar(task_id.as_str())
    );
    let complete_response = run_live_s04_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        complete_agent_name.as_str(),
        key_file.as_str(),
        "probe-complete-task",
        "complete_task",
        complete_arguments.as_str(),
    )?;
    let complete_state = json_optional_string_field(complete_response.as_str(), "state")
        .ok_or_else(|| {
            format!("mcp live s04 complete_task response missing state field: {complete_response}")
        })?;
    if complete_state.trim().is_empty() {
        return Err("mcp live s04 complete_task returned empty state".to_owned());
    }

    let release_arguments = format!(
        "{{\"escrow_id\":\"{}\"}}",
        escape_json_scalar(escrow_id.as_str())
    );
    let release_response = run_live_s04_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        release_agent_name.as_str(),
        key_file.as_str(),
        "probe-release-escrow",
        "release_escrow",
        release_arguments.as_str(),
    )?;
    let release_state =
        json_optional_string_field(release_response.as_str(), "state").ok_or_else(|| {
            format!("mcp live s04 release_escrow response missing state field: {release_response}")
        })?;
    if release_state.trim().is_empty() {
        return Err("mcp live s04 release_escrow returned empty state".to_owned());
    }

    Ok(())
}
