use super::{
    default_endpoint, env_payload, env_value, DEFAULT_S11_MESSAGE_PAYLOAD,
    DEFAULT_S11_PRIMARY_AGENT_NAME, DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD,
    DEFAULT_S11_STALE_MESSAGE_PAYLOAD,
};
use crate::drivers::cli_scripted::live_probe_tranche_three::live_probe_support::{
    query_message_status, send_message_with_validated_receipt,
};
use crate::drivers::shared_helpers::validate_s07_replay_reason_marker;

pub(super) fn run_live_s11_cli_signer_rotation_probe() -> Result<(), String> {
    let settings = s11_settings();
    let primary_message_id = send_primary_message(&settings)?;
    query_primary_message(&settings, primary_message_id.as_str())?;
    let rotated_message_id = send_rotated_message(&settings, primary_message_id.as_str())?;
    query_rotated_message(&settings, rotated_message_id.as_str())?;
    reject_stale_primary(&settings)
}

struct S11Settings {
    endpoint: String,
    primary_agent_name: String,
    rotated_agent_name: String,
    message_payload: String,
    rotated_message_payload: String,
    stale_message_payload: String,
}

fn s11_settings() -> S11Settings {
    let primary_agent_name = env_value(
        "KAMN_E2E_S11_PRIMARY_AGENT_NAME",
        DEFAULT_S11_PRIMARY_AGENT_NAME,
    );
    S11Settings {
        endpoint: default_endpoint(),
        rotated_agent_name: super::super::env_var_or_else(
            "KAMN_E2E_S11_ROTATED_AGENT_NAME",
            || format!("{primary_agent_name}-rotated"),
        ),
        message_payload: env_payload("KAMN_E2E_S11_MESSAGE_PAYLOAD", DEFAULT_S11_MESSAGE_PAYLOAD),
        rotated_message_payload: env_payload(
            "KAMN_E2E_S11_ROTATED_MESSAGE_PAYLOAD",
            DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD,
        ),
        stale_message_payload: env_payload(
            "KAMN_E2E_S11_STALE_MESSAGE_PAYLOAD",
            DEFAULT_S11_STALE_MESSAGE_PAYLOAD,
        ),
        primary_agent_name,
    }
}

fn send_primary_message(settings: &S11Settings) -> Result<String, String> {
    send_message_with_validated_receipt(
        settings.endpoint.as_str(),
        settings.primary_agent_name.as_str(),
        settings.message_payload.as_str(),
        "cli live s11 primary send-message",
    )
}

fn query_primary_message(settings: &S11Settings, message_id: &str) -> Result<(), String> {
    query_message_status(
        settings.endpoint.as_str(),
        format!("{}-query", settings.primary_agent_name).as_str(),
        message_id,
        "cli live s11 primary query-message",
    )
}

fn send_rotated_message(
    settings: &S11Settings,
    primary_message_id: &str,
) -> Result<String, String> {
    let rotated_message_id = send_message_with_validated_receipt(
        settings.endpoint.as_str(),
        settings.rotated_agent_name.as_str(),
        settings.rotated_message_payload.as_str(),
        "cli live s11 rotated send-message",
    )?;
    super::super::validate_s08_distinct_message_ids(
        primary_message_id,
        rotated_message_id.as_str(),
        "cli live s11 rotated send-message",
    )?;
    Ok(rotated_message_id)
}

fn query_rotated_message(settings: &S11Settings, message_id: &str) -> Result<(), String> {
    query_message_status(
        settings.endpoint.as_str(),
        format!("{}-query", settings.rotated_agent_name).as_str(),
        message_id,
        "cli live s11 rotated query-message",
    )
}

fn reject_stale_primary(settings: &S11Settings) -> Result<(), String> {
    let replay_error = super::super::run_cli_command_expect_failure_with_agent_name(
        super::cli_binary().as_str(),
        &[
            "send-message",
            "--endpoint",
            settings.endpoint.as_str(),
            "--format",
            "text",
            settings.stale_message_payload.as_str(),
        ],
        "cli live s11 stale-primary send-message",
        settings.primary_agent_name.as_str(),
    )?;
    validate_s07_replay_reason_marker(
        replay_error.as_str(),
        "cli live s11 stale-primary send-message",
    )
}
