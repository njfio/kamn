use super::super::{parse_text_output_field, run_cli_command_capture_stdout_with_agent_name};
use super::{cli_binary, validate_non_empty};

pub(super) fn send_message_with_status(
    endpoint: &str,
    agent_name: &str,
    payload: &str,
    step: &str,
) -> Result<String, String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint,
            "--format",
            "text",
            payload,
        ],
        step,
        agent_name,
    )?;
    validate_s08_message_receipt_fields(output.as_str(), step)
}

pub(super) fn query_message_status(
    endpoint: &str,
    agent_name: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint,
            "--format",
            "text",
            expected_message_id,
        ],
        step,
        agent_name,
    )?;
    validate_s08_query_message_response(output.as_str(), expected_message_id, step)
}

pub(super) fn require_health_status(
    endpoint: &str,
    agent_name: &str,
    step: &str,
) -> Result<(), String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &["health", "--endpoint", endpoint, "--format", "text"],
        step,
        agent_name,
    )?;
    validate_non_empty(
        require_field(output.as_str(), "status", step)?,
        &format!("{step} returned empty status"),
    )
}

pub(crate) fn validate_s08_message_receipt_fields(
    output: &str,
    step: &str,
) -> Result<String, String> {
    let message_id = require_field(output, "message_id", step)?;
    validate_non_empty(message_id, &format!("{step} returned empty message_id"))?;
    validate_non_empty(
        require_field(output, "status", step)?,
        &format!("{step} returned empty status"),
    )?;
    Ok(message_id.to_owned())
}

pub(crate) fn validate_s08_query_message_response(
    output: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let observed_message_id = require_field(output, "message_id", step)?;
    if observed_message_id != expected_message_id {
        return Err(format!(
            "{step} returned mismatched message_id: expected={expected_message_id}, got={observed_message_id}"
        ));
    }
    validate_non_empty(
        require_field(output, "status", step)?,
        &format!("{step} returned empty status"),
    )
}

pub(crate) fn validate_s08_distinct_message_ids(
    pre_message_id: &str,
    post_message_id: &str,
    step: &str,
) -> Result<(), String> {
    if post_message_id == pre_message_id {
        return Err(format!("{step} returned duplicate message_id"));
    }
    Ok(())
}

fn require_field<'a>(output: &'a str, key: &str, step: &str) -> Result<&'a str, String> {
    parse_text_output_field(output, key)
        .ok_or_else(|| format!("{step} response missing {key} field: {output}"))
}
