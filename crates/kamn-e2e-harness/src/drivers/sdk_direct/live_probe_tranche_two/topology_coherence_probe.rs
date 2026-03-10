use super::{kolme_endpoint, DEFAULT_S10_AGENT_NAME, DEFAULT_S10_MESSAGE_PAYLOAD};
use crate::drivers::sdk_direct::live_probe_tranche_two::message_query_support::{
    query_message_status, require_health_status, send_message_with_status,
};

pub(super) fn run_live_s10_topology_coherence_probe() -> Result<(), String> {
    let settings = s10_settings();
    let message_id = send_primary_message(&settings)?;
    query_secondary_message(&settings, message_id.as_str())?;
    query_tertiary_message(&settings, message_id.as_str())?;
    require_secondary_health(&settings)?;
    require_tertiary_health(&settings)
}

struct S10Settings {
    primary_endpoint: String,
    secondary_endpoint: String,
    tertiary_endpoint: String,
    kolme_endpoint: String,
    base_agent_name: String,
    message_payload: String,
}

fn s10_settings() -> S10Settings {
    let primary_endpoint =
        super::super::env_var_or_else("KAMN_E2E_S10_PRIMARY_ENDPOINT", super::default_endpoint);
    let secondary_endpoint =
        super::super::env_var_or_else("KAMN_E2E_S10_SECONDARY_ENDPOINT", || {
            primary_endpoint.clone()
        });
    S10Settings {
        tertiary_endpoint: super::super::env_var_or_else("KAMN_E2E_S10_TERTIARY_ENDPOINT", || {
            secondary_endpoint.clone()
        }),
        kolme_endpoint: kolme_endpoint(),
        base_agent_name: super::super::env_var_or_default(
            "KAMN_E2E_S10_AGENT_NAME",
            DEFAULT_S10_AGENT_NAME,
        ),
        message_payload: super::super::env_var_or_default(
            "KAMN_E2E_S10_MESSAGE_PAYLOAD",
            DEFAULT_S10_MESSAGE_PAYLOAD,
        ),
        primary_endpoint,
        secondary_endpoint,
    }
}

fn send_primary_message(settings: &S10Settings) -> Result<String, String> {
    send_message_with_status(
        settings.primary_endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-primary-send", settings.base_agent_name).as_str(),
        settings.message_payload.as_str(),
        "sdk-direct live s10 primary connect failed",
        "sdk-direct live s10 primary send-message",
    )
}

fn query_secondary_message(settings: &S10Settings, message_id: &str) -> Result<(), String> {
    query_message_status(
        settings.secondary_endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-secondary-query", settings.base_agent_name).as_str(),
        message_id,
        "sdk-direct live s10 secondary connect failed",
        "sdk-direct live s10 secondary query-message",
    )
}

fn query_tertiary_message(settings: &S10Settings, message_id: &str) -> Result<(), String> {
    query_message_status(
        settings.tertiary_endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-tertiary-query", settings.base_agent_name).as_str(),
        message_id,
        "sdk-direct live s10 tertiary connect failed",
        "sdk-direct live s10 tertiary query-message",
    )
}

fn require_secondary_health(settings: &S10Settings) -> Result<(), String> {
    require_health_status(
        settings.secondary_endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-secondary-boundary", settings.base_agent_name).as_str(),
        "sdk-direct live s10 secondary connect failed",
        "sdk-direct live s10 secondary health check",
    )
}

fn require_tertiary_health(settings: &S10Settings) -> Result<(), String> {
    require_health_status(
        settings.tertiary_endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-tertiary-boundary", settings.base_agent_name).as_str(),
        "sdk-direct live s10 tertiary connect failed",
        "sdk-direct live s10 tertiary health check",
    )
}
