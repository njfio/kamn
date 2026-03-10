use super::super::{
    parse_text_output_field, run_cli_command_capture_stdout_with_agent_name,
    AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_ENV, AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_VALUE,
    DEFAULT_S02_AGENT_NAME, DEFAULT_S02_MESSAGE_PAYLOAD, DEFAULT_S02_REPLY_PAYLOAD,
};
use super::{cli_binary, endpoint, env_payload, validate_non_empty};
use std::process::{Command, ExitStatus, Stdio};

pub(super) fn run_live_s01_cli_health_probe() -> Result<(), String> {
    let status = health_probe_status()?;
    if status.success() {
        return Ok(());
    }
    Err(format!(
        "cli live health probe failed (exit_status={})",
        exit_status_label(status)
    ))
}

pub(super) fn run_live_s02_cli_direct_message_probe() -> Result<(), String> {
    let agent_name = super::super::env_var_or_default("KAMN_AGENT_NAME", DEFAULT_S02_AGENT_NAME);
    let message_payload = env_payload("KAMN_E2E_S02_MESSAGE_PAYLOAD", DEFAULT_S02_MESSAGE_PAYLOAD);
    let reply_payload = env_payload("KAMN_E2E_S02_REPLY_PAYLOAD", DEFAULT_S02_REPLY_PAYLOAD);
    run_message_roundtrip(agent_name.as_str(), "send", message_payload.as_str())?;
    run_message_roundtrip(agent_name.as_str(), "reply", reply_payload.as_str())
}

fn run_message_roundtrip(agent_name: &str, suffix: &str, payload: &str) -> Result<(), String> {
    let message_id = send_message_receipt(agent_name, suffix, payload)?;
    query_message_receipt(agent_name, suffix, message_id.as_str())
}

fn health_probe_status() -> Result<ExitStatus, String> {
    Command::new(cli_binary())
        .arg("health")
        .arg("--endpoint")
        .arg(endpoint())
        .arg("--format")
        .arg("text")
        .env(
            AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_ENV,
            AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_VALUE,
        )
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("cli live health probe failed to spawn: {error}"))
}

fn send_message_receipt(agent_name: &str, suffix: &str, payload: &str) -> Result<String, String> {
    let output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            payload,
        ],
        send_step(suffix),
        format!("{agent_name}-{suffix}").as_str(),
    )?;
    let message_id = require_field(output.as_str(), "message_id", send_step(suffix))?;
    validate_non_empty(
        require_field(output.as_str(), "status", send_step(suffix))?,
        &format!("{} returned empty status", send_step(suffix)),
    )?;
    Ok(message_id.to_owned())
}

fn query_message_receipt(agent_name: &str, suffix: &str, message_id: &str) -> Result<(), String> {
    let step = query_step(suffix);
    let output = query_message_output(agent_name, suffix, message_id, step)?;
    validate_matching_message_id(output.as_str(), message_id, step)?;
    validate_non_empty(
        require_field(output.as_str(), "status", step)?,
        &format!("{step} returned empty status"),
    )
}

fn query_message_output(
    agent_name: &str,
    suffix: &str,
    message_id: &str,
    step: &str,
) -> Result<String, String> {
    let query_suffix = if suffix == "send" {
        "query"
    } else {
        "query-reply"
    };
    run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint().as_str(),
            "--format",
            "text",
            message_id,
        ],
        step,
        format!("{agent_name}-{query_suffix}").as_str(),
    )
}

fn validate_matching_message_id(output: &str, message_id: &str, step: &str) -> Result<(), String> {
    let queried_message_id = require_field(output, "message_id", step)?;
    if queried_message_id == message_id {
        return Ok(());
    }
    Err(format!(
        "{step} returned mismatched message_id: expected={message_id}, got={queried_message_id}"
    ))
}

fn require_field<'a>(output: &'a str, key: &str, step: &str) -> Result<&'a str, String> {
    parse_text_output_field(output, key)
        .ok_or_else(|| format!("{step} response missing {key} field: {output}"))
}

fn send_step(suffix: &str) -> &'static str {
    if suffix == "send" {
        "cli live s02 send-message"
    } else {
        "cli live s02 reply send-message"
    }
}

fn query_step(suffix: &str) -> &'static str {
    if suffix == "send" {
        "cli live s02 query-message"
    } else {
        "cli live s02 reply query-message"
    }
}

fn exit_status_label(status: ExitStatus) -> String {
    status
        .code()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "signal".to_owned())
}
