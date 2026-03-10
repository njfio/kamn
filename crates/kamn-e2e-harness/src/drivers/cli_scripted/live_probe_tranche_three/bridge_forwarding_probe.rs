use super::{
    default_endpoint, env_payload, env_value, validate_non_empty, DEFAULT_S13_AGENT_NAME,
    DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD,
};
use crate::drivers::cli_scripted::live_probe_tranche_three::live_probe_support::validate_bridge_forward_fields;
use crate::drivers::shared_helpers::{
    validate_s13_bridge_field_coherence, validate_s13_bridge_id_match,
};

pub(super) fn run_live_s13_cli_bridge_forwarding_probe() -> Result<(), String> {
    let settings = s13_settings();
    let bridge_id = submit_bridge_message(&settings)?;
    let forwarded = forward_bridge_message(&settings, bridge_id.as_str())?;
    query_bridge_message(&settings, bridge_id.as_str(), &forwarded)
}

struct S13Settings {
    endpoint: String,
    base_agent_name: String,
    submit_payload: String,
}

struct S13ForwardedState {
    bridge_status: String,
    target_message_id: String,
    forward_tx_hash: String,
}

fn s13_settings() -> S13Settings {
    S13Settings {
        endpoint: default_endpoint(),
        base_agent_name: env_value("KAMN_E2E_S13_AGENT_NAME", DEFAULT_S13_AGENT_NAME),
        submit_payload: env_payload(
            "KAMN_E2E_S13_SUBMIT_BRIDGE_PAYLOAD",
            DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD,
        ),
    }
}

fn submit_bridge_message(settings: &S13Settings) -> Result<String, String> {
    let output = run_bridge_command(
        settings,
        "submit-bridge-message",
        settings.submit_payload.as_str(),
        "submit",
    )?;
    let bridge_id = require_field(
        output.as_str(),
        "bridge_id",
        "cli live s13 submit-bridge-message",
    )?;
    validate_non_empty(
        bridge_id,
        "cli live s13 submit-bridge-message returned empty bridge_id",
    )?;
    validate_non_empty(
        require_field(
            output.as_str(),
            "source_message_id",
            "cli live s13 submit-bridge-message",
        )?,
        "cli live s13 submit-bridge-message returned empty source_message_id",
    )?;
    validate_non_empty(
        require_field(
            output.as_str(),
            "bridge_status",
            "cli live s13 submit-bridge-message",
        )?,
        "cli live s13 submit-bridge-message returned empty bridge_status",
    )?;
    Ok(bridge_id.to_owned())
}

fn forward_bridge_message(
    settings: &S13Settings,
    bridge_id: &str,
) -> Result<S13ForwardedState, String> {
    let output = run_bridge_command(settings, "forward-bridge-message", bridge_id, "forward")?;
    validate_s13_bridge_id_match(
        bridge_id,
        require_field(
            output.as_str(),
            "bridge_id",
            "cli live s13 forward-bridge-message",
        )?,
        "cli live s13 forward-bridge-message",
    )?;
    let state = forwarded_state(output.as_str(), "cli live s13 forward-bridge-message")?;
    validate_bridge_forward_fields(
        state.bridge_status.as_str(),
        state.target_message_id.as_str(),
        state.forward_tx_hash.as_str(),
        "cli live s13 forward-bridge-message",
    )?;
    Ok(state)
}

fn query_bridge_message(
    settings: &S13Settings,
    bridge_id: &str,
    forwarded: &S13ForwardedState,
) -> Result<(), String> {
    let output = run_bridge_command(settings, "query-bridge-message", bridge_id, "query")?;
    validate_s13_bridge_id_match(
        bridge_id,
        require_field(
            output.as_str(),
            "bridge_id",
            "cli live s13 query-bridge-message",
        )?,
        "cli live s13 query-bridge-message",
    )?;
    let queried = forwarded_state(output.as_str(), "cli live s13 query-bridge-message")?;
    validate_queried_field(
        forwarded.bridge_status.as_str(),
        queried.bridge_status.as_str(),
        "bridge_status",
    )?;
    validate_queried_field(
        forwarded.target_message_id.as_str(),
        queried.target_message_id.as_str(),
        "target_message_id",
    )?;
    validate_queried_field(
        forwarded.forward_tx_hash.as_str(),
        queried.forward_tx_hash.as_str(),
        "forward_tx_hash",
    )
}

fn run_bridge_command(
    settings: &S13Settings,
    command: &str,
    value: &str,
    suffix: &str,
) -> Result<String, String> {
    super::super::run_cli_command_capture_stdout_with_agent_name(
        super::cli_binary().as_str(),
        &[
            command,
            "--endpoint",
            settings.endpoint.as_str(),
            "--format",
            "text",
            value,
        ],
        &format!("cli live s13 {command}"),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
    )
}

fn forwarded_state(output: &str, step: &str) -> Result<S13ForwardedState, String> {
    Ok(S13ForwardedState {
        bridge_status: require_field(output, "bridge_status", step)?.to_owned(),
        target_message_id: require_field(output, "target_message_id", step)?.to_owned(),
        forward_tx_hash: require_field(output, "forward_tx_hash", step)?.to_owned(),
    })
}

fn validate_queried_field(expected: &str, observed: &str, field: &str) -> Result<(), String> {
    validate_s13_bridge_field_coherence(
        expected,
        observed,
        field,
        "cli live s13 query-bridge-message",
    )
}

fn require_field<'a>(output: &'a str, key: &str, step: &str) -> Result<&'a str, String> {
    super::super::parse_text_output_field(output, key)
        .ok_or_else(|| format!("{step} response missing {key} field: {output}"))
}
