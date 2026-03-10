use std::env;

use super::super::{
    live_s07_probe_agent_suffix, parse_text_output_field, run_cli_command_capture_stdout,
    run_cli_command_expect_failure_with_agent_name,
};
use super::{
    agent_name, cli_binary, default_endpoint, env_payload, validate_non_empty,
    DEFAULT_S06_BLOCK_HEIGHT, DEFAULT_S06_FINALITY, DEFAULT_S06_MESSAGE_ID, DEFAULT_S06_TX_HASH,
    DEFAULT_S07_AGENT_NAME, DEFAULT_S07_MESSAGE_PAYLOAD,
};
use crate::drivers::shared_helpers::validate_s07_replay_reason_marker;

pub(super) fn run_live_s06_cli_proof_verification_probe() -> Result<(), String> {
    let settings = s06_settings()?;
    let output = s06_verify_output(&settings)?;
    validate_s06_verify_output(output.as_str())
}

pub(super) fn run_live_s07_cli_replay_protection_probe() -> Result<(), String> {
    let settings = s07_settings();
    send_s07_initial_message(&settings)?;
    reject_s07_replay(&settings)
}

struct S06Settings {
    endpoint: String,
    message_id: String,
    tx_hash: String,
    block_height: String,
    finality: String,
}

struct S07Settings {
    endpoint: String,
    agent_name: String,
    payload: String,
}

fn s06_settings() -> Result<S06Settings, String> {
    Ok(S06Settings {
        endpoint: default_endpoint(),
        message_id: env_payload("KAMN_E2E_S06_PROOF_MESSAGE_ID", DEFAULT_S06_MESSAGE_ID),
        tx_hash: agent_name("KAMN_E2E_S06_PROOF_TX_HASH", DEFAULT_S06_TX_HASH),
        block_height: s06_block_height()?.to_string(),
        finality: agent_name("KAMN_E2E_S06_PROOF_FINALITY", DEFAULT_S06_FINALITY),
    })
}

fn s07_settings() -> S07Settings {
    let base_agent_name = agent_name("KAMN_E2E_S07_AGENT_NAME", DEFAULT_S07_AGENT_NAME);
    S07Settings {
        endpoint: default_endpoint(),
        agent_name: format!(
            "{base_agent_name}-{}",
            live_s07_probe_agent_suffix().as_str()
        ),
        payload: env_payload("KAMN_E2E_S07_REPLAY_PAYLOAD", DEFAULT_S07_MESSAGE_PAYLOAD),
    }
}

fn s06_block_height() -> Result<u64, String> {
    env::var("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("cli live s06 invalid block height env value: {raw}"))
        })
        .transpose()
        .map(|value| value.unwrap_or(DEFAULT_S06_BLOCK_HEIGHT))
}

fn s06_verify_output(settings: &S06Settings) -> Result<String, String> {
    run_cli_command_capture_stdout(
        cli_binary().as_str(),
        &[
            "verify-proof",
            "--endpoint",
            settings.endpoint.as_str(),
            "--format",
            "text",
            settings.message_id.as_str(),
            settings.tx_hash.as_str(),
            settings.block_height.as_str(),
            settings.finality.as_str(),
        ],
        "cli live s06 verify-proof",
    )
}

fn validate_s06_verify_output(output: &str) -> Result<(), String> {
    require_s06_verified(output)?;
    require_s06_finality(output)?;
    require_s06_block_height(output)
}

fn require_s06_verified(output: &str) -> Result<(), String> {
    let verified = require_field(output, "verified", "cli live s06 verify-proof")?;
    if verified == "true" {
        return Ok(());
    }
    Err(format!(
        "cli live s06 verify-proof returned verified={verified}"
    ))
}

fn require_s06_finality(output: &str) -> Result<(), String> {
    let finality = require_field(output, "finality", "cli live s06 verify-proof")?;
    if finality == "FINAL" {
        return Ok(());
    }
    Err(format!(
        "cli live s06 verify-proof returned non-final finality: {finality}"
    ))
}

fn require_s06_block_height(output: &str) -> Result<(), String> {
    let block_height = require_field(output, "block_height", "cli live s06 verify-proof")?;
    let parsed_height = block_height.parse::<u64>().map_err(|_| {
        format!("cli live s06 verify-proof returned invalid block_height: {output}")
    })?;
    if parsed_height != 0 {
        return Ok(());
    }
    Err("cli live s06 verify-proof returned block_height=0".to_owned())
}

fn send_s07_initial_message(settings: &S07Settings) -> Result<(), String> {
    let output =
        run_cli_command_capture_stdout_with_agent(settings, "cli live s07 initial send-message")?;
    let message_id = require_field(
        output.as_str(),
        "message_id",
        "cli live s07 initial send-message",
    )?;
    validate_non_empty(
        message_id,
        "cli live s07 initial send-message returned empty message_id",
    )?;
    validate_non_empty(
        require_field(
            output.as_str(),
            "status",
            "cli live s07 initial send-message",
        )?,
        "cli live s07 initial send-message returned empty status",
    )
}

fn reject_s07_replay(settings: &S07Settings) -> Result<(), String> {
    let replay_error = run_cli_command_expect_failure_with_agent_name(
        cli_binary().as_str(),
        &[
            "send-message",
            "--endpoint",
            settings.endpoint.as_str(),
            "--format",
            "text",
            settings.payload.as_str(),
        ],
        "cli live s07 replay send-message",
        settings.agent_name.as_str(),
    )?;
    validate_s07_replay_reason_marker(replay_error.as_str(), "cli live s07 replay send-message")
}

fn run_cli_command_capture_stdout_with_agent(
    settings: &S07Settings,
    step: &str,
) -> Result<String, String> {
    super::super::run_cli_command_capture_stdout_with_agent_name(
        cli_binary().as_str(),
        &[
            "send-message",
            "--endpoint",
            settings.endpoint.as_str(),
            "--format",
            "text",
            settings.payload.as_str(),
        ],
        step,
        settings.agent_name.as_str(),
    )
}

fn require_field<'a>(output: &'a str, key: &str, step: &str) -> Result<&'a str, String> {
    parse_text_output_field(output, key)
        .ok_or_else(|| format!("{step} response missing {key} field: {output}"))
}
