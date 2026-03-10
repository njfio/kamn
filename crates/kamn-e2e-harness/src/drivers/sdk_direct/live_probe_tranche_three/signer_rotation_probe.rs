use super::{
    default_endpoint, kolme_endpoint, DEFAULT_S11_MESSAGE_PAYLOAD, DEFAULT_S11_PRIMARY_AGENT_NAME,
    DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD, DEFAULT_S11_STALE_MESSAGE_PAYLOAD,
};
use crate::drivers::sdk_direct::live_probe_tranche_three::live_probe_support::{
    query_message_status, send_message_with_validated_receipt,
};

pub(super) fn run_live_s11_signer_rotation_probe() -> Result<(), String> {
    let settings = s11_settings();
    let primary_message_id = send_primary_message(&settings)?;
    query_primary_message(&settings, primary_message_id.as_str())?;
    let rotated_message_id = send_rotated_message(&settings, primary_message_id.as_str())?;
    query_rotated_message(&settings, rotated_message_id.as_str())?;
    reject_stale_primary(&settings)
}

struct S11Settings {
    endpoint: String,
    kolme_endpoint: String,
    primary_agent_name: String,
    rotated_agent_name: String,
    message_payload: String,
    rotated_message_payload: String,
    stale_message_payload: String,
}

fn s11_settings() -> S11Settings {
    let primary_agent_name = s11_primary_agent_name();
    S11Settings {
        endpoint: default_endpoint(),
        kolme_endpoint: kolme_endpoint(),
        rotated_agent_name: super::super::env_var_or_else(
            "KAMN_E2E_S11_ROTATED_AGENT_NAME",
            || format!("{primary_agent_name}-rotated"),
        ),
        message_payload: s11_message_payload(),
        rotated_message_payload: s11_rotated_message_payload(),
        stale_message_payload: s11_stale_message_payload(),
        primary_agent_name,
    }
}

fn send_primary_message(settings: &S11Settings) -> Result<String, String> {
    send_message_with_validated_receipt(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        settings.primary_agent_name.as_str(),
        settings.message_payload.as_str(),
        "sdk-direct live s11 primary connect failed",
        "sdk-direct live s11 primary send-message",
    )
}

fn query_primary_message(settings: &S11Settings, message_id: &str) -> Result<(), String> {
    query_message_status(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-query", settings.primary_agent_name).as_str(),
        message_id,
        "sdk-direct live s11 primary query connect failed",
        "sdk-direct live s11 primary query-message",
    )
}

fn send_rotated_message(
    settings: &S11Settings,
    primary_message_id: &str,
) -> Result<String, String> {
    let rotated_message_id = send_message_with_validated_receipt(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        settings.rotated_agent_name.as_str(),
        settings.rotated_message_payload.as_str(),
        "sdk-direct live s11 rotated connect failed",
        "sdk-direct live s11 rotated send-message",
    )?;
    super::super::validate_s08_distinct_message_ids(
        primary_message_id,
        rotated_message_id.as_str(),
        "sdk-direct live s11 rotated send-message",
    )?;
    Ok(rotated_message_id)
}

fn query_rotated_message(settings: &S11Settings, message_id: &str) -> Result<(), String> {
    query_message_status(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-query", settings.rotated_agent_name).as_str(),
        message_id,
        "sdk-direct live s11 rotated query connect failed",
        "sdk-direct live s11 rotated query-message",
    )
}

fn reject_stale_primary(settings: &S11Settings) -> Result<(), String> {
    let handle = super::connect_agent(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        settings.primary_agent_name.as_str(),
        "sdk-direct live s11 stale-primary connect failed",
    )?;
    let stale_primary_error = handle
        .send_message(settings.stale_message_payload.as_str())
        .err()
        .ok_or_else(|| {
            "sdk-direct live s11 stale-primary send-message unexpectedly succeeded".to_owned()
        })?;
    super::super::validate_s07_replay_reason_marker(
        stale_primary_error.to_string().as_str(),
        "sdk-direct live s11 stale-primary send-message",
    )
}

fn s11_primary_agent_name() -> String {
    super::super::env_var_or_default(
        "KAMN_E2E_S11_PRIMARY_AGENT_NAME",
        DEFAULT_S11_PRIMARY_AGENT_NAME,
    )
}

fn s11_message_payload() -> String {
    super::super::env_var_or_default("KAMN_E2E_S11_MESSAGE_PAYLOAD", DEFAULT_S11_MESSAGE_PAYLOAD)
}

fn s11_rotated_message_payload() -> String {
    super::super::env_var_or_default(
        "KAMN_E2E_S11_ROTATED_MESSAGE_PAYLOAD",
        DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD,
    )
}

fn s11_stale_message_payload() -> String {
    super::super::env_var_or_default(
        "KAMN_E2E_S11_STALE_MESSAGE_PAYLOAD",
        DEFAULT_S11_STALE_MESSAGE_PAYLOAD,
    )
}
