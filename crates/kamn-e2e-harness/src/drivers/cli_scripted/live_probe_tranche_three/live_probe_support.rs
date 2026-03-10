use super::super::{parse_text_output_field, run_cli_command_capture_stdout_with_agent_name};
use super::{cli_binary, validate_non_empty};

pub(super) fn send_message_with_validated_receipt(
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
    super::super::validate_s08_message_receipt_fields(output.as_str(), step)
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
    super::super::validate_s08_query_message_response(output.as_str(), expected_message_id, step)
}

pub(crate) fn validate_s14_cli_verify_proof_response(
    output: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    validate_s14_message_id(output, expected_message_id, step)?;
    validate_s14_verified(output, step)?;
    validate_s14_finality(output, step)?;
    validate_s14_block_height(output, step)
}

pub(super) fn validate_content_state(
    lifecycle_state: &str,
    redaction_status: &str,
    step: &str,
) -> Result<(), String> {
    validate_non_empty(
        lifecycle_state,
        &format!("{step} returned empty lifecycle_state"),
    )?;
    validate_non_empty(
        redaction_status,
        &format!("{step} returned empty redaction_status"),
    )
}

pub(super) fn validate_bridge_forward_fields(
    bridge_status: &str,
    target_message_id: &str,
    forward_tx_hash: &str,
    step: &str,
) -> Result<(), String> {
    validate_non_empty(
        bridge_status,
        &format!("{step} returned empty bridge_status"),
    )?;
    validate_non_empty(
        target_message_id,
        &format!("{step} returned empty target_message_id"),
    )?;
    validate_non_empty(
        forward_tx_hash,
        &format!("{step} returned empty forward_tx_hash"),
    )
}

fn validate_s14_message_id(
    output: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let observed = require_field(output, "message_id", step)?;
    if observed == expected_message_id {
        return Ok(());
    }
    Err(format!(
        "{step} returned mismatched message_id: expected={expected_message_id}, got={observed}"
    ))
}

fn validate_s14_verified(output: &str, step: &str) -> Result<(), String> {
    let observed = require_field(output, "verified", step)?;
    if observed == "true" {
        return Ok(());
    }
    Err(format!("{step} returned verified={observed}"))
}

fn validate_s14_finality(output: &str, step: &str) -> Result<(), String> {
    let observed = require_field(output, "finality", step)?;
    if observed == "FINAL" {
        return Ok(());
    }
    Err(format!("{step} returned non-final finality: {observed}"))
}

fn validate_s14_block_height(output: &str, step: &str) -> Result<(), String> {
    let observed = require_field(output, "block_height", step)?;
    let parsed = observed
        .parse::<u64>()
        .map_err(|_| format!("{step} returned invalid block_height: {output}"))?;
    if parsed != 0 {
        return Ok(());
    }
    Err(format!("{step} returned block_height=0"))
}

fn require_field<'a>(output: &'a str, key: &str, step: &str) -> Result<&'a str, String> {
    parse_text_output_field(output, key)
        .ok_or_else(|| format!("{step} response missing {key} field: {output}"))
}
