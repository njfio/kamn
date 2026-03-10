use super::super::super::{
    parse_text_output_field, run_cli_command_capture_stdout_with_agent_name,
    DEFAULT_S04_AGENT_NAME, DEFAULT_S04_CREATE_TASK_PAYLOAD, DEFAULT_S04_ESCROW_AMOUNT,
};
use super::super::{cli_binary, endpoint, validate_non_empty};

pub(super) fn run_live_s04_cli_task_lifecycle_probe() -> Result<(), String> {
    let agent_name =
        super::super::super::env_var_or_default("KAMN_AGENT_NAME", DEFAULT_S04_AGENT_NAME);
    let task_id = create_task(agent_name.as_str())?;
    let escrow_id = fund_escrow(agent_name.as_str(), task_id.as_str())?;
    require_state(
        agent_name.as_str(),
        "accept-task",
        task_id.as_str(),
        "accept",
    )?;
    require_state(
        agent_name.as_str(),
        "complete-task",
        task_id.as_str(),
        "complete",
    )?;
    require_state(
        agent_name.as_str(),
        "release-escrow",
        escrow_id.as_str(),
        "release",
    )
}

fn create_task(agent_name: &str) -> Result<String, String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "create-task",
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            super::super::env_payload(
                "KAMN_E2E_S04_CREATE_TASK_PAYLOAD",
                DEFAULT_S04_CREATE_TASK_PAYLOAD,
            )
            .as_str(),
        ],
        "cli live s04 create-task",
        format!("{agent_name}-create").as_str(),
    )?;
    let task_id = require_field(output.as_str(), "task_id", "cli live s04 create-task")?;
    validate_non_empty(task_id, "cli live s04 create-task returned empty task_id")?;
    Ok(task_id.to_owned())
}

fn fund_escrow(agent_name: &str, task_id: &str) -> Result<String, String> {
    let payload = format!("{{\"task_id\":\"{task_id}\",\"amount\":{DEFAULT_S04_ESCROW_AMOUNT}}}");
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "fund-escrow",
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            payload.as_str(),
        ],
        "cli live s04 fund-escrow",
        format!("{agent_name}-fund").as_str(),
    )?;
    let escrow_id = require_field(output.as_str(), "escrow_id", "cli live s04 fund-escrow")?;
    validate_non_empty(
        escrow_id,
        "cli live s04 fund-escrow returned empty escrow_id",
    )?;
    Ok(escrow_id.to_owned())
}

fn require_state(agent_name: &str, command: &str, id: &str, suffix: &str) -> Result<(), String> {
    let step = match command {
        "accept-task" => "cli live s04 accept-task",
        "complete-task" => "cli live s04 complete-task",
        _ => "cli live s04 release-escrow",
    };
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            command,
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            id,
        ],
        step,
        format!("{agent_name}-{suffix}").as_str(),
    )?;
    validate_non_empty(
        require_field(output.as_str(), "state", step)?,
        &format!("{step} returned empty state"),
    )
}

fn require_field<'a>(output: &'a str, key: &str, step: &str) -> Result<&'a str, String> {
    parse_text_output_field(output, key)
        .ok_or_else(|| format!("{step} response missing {key} field: {output}"))
}
