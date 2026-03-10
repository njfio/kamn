use super::super::*;
use std::env;

use crate::drivers::mcp_agent::live_probe_tranche_two::message_query_support::{
    payload_arguments, require_non_empty, required_string_field,
};

pub(crate) fn run_live_s04_mcp_task_lifecycle_probe() -> Result<(), String> {
    let settings = s04_settings();
    let task_id = create_task(&settings)?;
    let escrow_id = fund_escrow(&settings, task_id.as_str())?;
    run_state_call(
        &settings,
        "accept_task",
        "probe-accept-task",
        task_id.as_str(),
        "task_id",
    )?;
    run_state_call(
        &settings,
        "complete_task",
        "probe-complete-task",
        task_id.as_str(),
        "task_id",
    )?;
    run_state_call(
        &settings,
        "release_escrow",
        "probe-release-escrow",
        escrow_id.as_str(),
        "escrow_id",
    )
}

struct S04Settings {
    binary: String,
    endpoint: String,
    key_file: String,
    create_agent_name: String,
    fund_agent_name: String,
    accept_agent_name: String,
    complete_agent_name: String,
    release_agent_name: String,
    create_payload: String,
}

fn s04_settings() -> S04Settings {
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    S04Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        create_agent_name: format!("{agent_name}-s04-create"),
        fund_agent_name: format!("{agent_name}-s04-fund"),
        accept_agent_name: format!("{agent_name}-s04-accept"),
        complete_agent_name: format!("{agent_name}-s04-complete"),
        release_agent_name: format!("{agent_name}-s04-release"),
        create_payload: env::var("KAMN_E2E_S04_CREATE_TASK_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S04_CREATE_TASK_PAYLOAD.to_owned()),
    }
}

fn create_task(settings: &S04Settings) -> Result<String, String> {
    let response = run_live_s04_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        settings.create_agent_name.as_str(),
        settings.key_file.as_str(),
        "probe-create-task",
        "create_task",
        payload_arguments(settings.create_payload.as_str()).as_str(),
    )?;
    let task_id = required_string_field(response.as_str(), "task_id", "mcp live s04 create_task")?;
    require_non_empty(task_id.as_str(), "mcp live s04 create_task", "task_id")?;
    Ok(task_id)
}

fn fund_escrow(settings: &S04Settings, task_id: &str) -> Result<String, String> {
    let payload = format!(
        "{{\"task_id\":\"{}\",\"amount\":{}}}",
        escape_json_scalar(task_id),
        DEFAULT_S04_ESCROW_AMOUNT
    );
    let response = run_live_s04_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        settings.fund_agent_name.as_str(),
        settings.key_file.as_str(),
        "probe-fund-escrow",
        "fund_escrow",
        payload_arguments(payload.as_str()).as_str(),
    )?;
    let escrow_id =
        required_string_field(response.as_str(), "escrow_id", "mcp live s04 fund_escrow")?;
    require_non_empty(escrow_id.as_str(), "mcp live s04 fund_escrow", "escrow_id")?;
    Ok(escrow_id)
}

fn run_state_call(
    settings: &S04Settings,
    tool_name: &str,
    request_id: &str,
    value: &str,
    field_name: &str,
) -> Result<(), String> {
    let agent_name = match tool_name {
        "accept_task" => settings.accept_agent_name.as_str(),
        "complete_task" => settings.complete_agent_name.as_str(),
        _ => settings.release_agent_name.as_str(),
    };
    let response = run_live_s04_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        agent_name,
        settings.key_file.as_str(),
        request_id,
        tool_name,
        format!("{{\"{field_name}\":\"{}\"}}", escape_json_scalar(value)).as_str(),
    )?;
    let step = format!("mcp live s04 {tool_name}");
    let state = required_string_field(response.as_str(), "state", step.as_str())?;
    require_non_empty(state.as_str(), step.as_str(), "state")
}
