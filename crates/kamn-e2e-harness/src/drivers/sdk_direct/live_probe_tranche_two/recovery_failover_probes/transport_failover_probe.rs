use super::{s09_settings, S09Settings};
use crate::drivers::sdk_direct::live_probe_tranche_two::message_query_support::{
    query_message_status, require_health_status, send_message_with_status,
};
use crate::drivers::sdk_direct::live_probe_tranche_two::validate_s08_distinct_message_ids;

pub(super) fn run_live_s09_transport_failover_probe() -> Result<(), String> {
    let settings = s09_settings();
    let pre_message_id = send_s09_message(
        &settings.primary_endpoint,
        &settings,
        "pre-send",
        settings.pre_message_payload.as_str(),
        "sdk-direct live s09 connect failed",
        "sdk-direct live s09 pre-failover send-message",
    )?;
    query_s09_message(
        &settings.primary_endpoint,
        &settings,
        "pre-query",
        pre_message_id.as_str(),
        "sdk-direct live s09 connect failed",
        "sdk-direct live s09 pre-failover query-message",
    )?;
    require_s09_health(&settings, "boundary")?;
    let post_message_id = send_s09_message(
        &settings.failover_endpoint,
        &settings,
        "post-send",
        settings.post_message_payload.as_str(),
        "sdk-direct live s09 failover connect failed",
        "sdk-direct live s09 post-failover send-message",
    )?;
    validate_s08_distinct_message_ids(
        pre_message_id.as_str(),
        post_message_id.as_str(),
        "sdk-direct live s09 post-failover send-message",
    )?;
    query_s09_message(
        &settings.failover_endpoint,
        &settings,
        "post-query",
        post_message_id.as_str(),
        "sdk-direct live s09 failover connect failed",
        "sdk-direct live s09 post-failover query-message",
    )
}

fn send_s09_message(
    endpoint: &str,
    settings: &S09Settings,
    suffix: &str,
    payload: &str,
    connect_context: &str,
    step: &str,
) -> Result<String, String> {
    send_message_with_status(
        endpoint,
        settings.kolme_endpoint.as_str(),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
        payload,
        connect_context,
        step,
    )
}

fn query_s09_message(
    endpoint: &str,
    settings: &S09Settings,
    suffix: &str,
    message_id: &str,
    connect_context: &str,
    step: &str,
) -> Result<(), String> {
    query_message_status(
        endpoint,
        settings.kolme_endpoint.as_str(),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
        message_id,
        connect_context,
        step,
    )
}

fn require_s09_health(settings: &S09Settings, suffix: &str) -> Result<(), String> {
    require_health_status(
        settings.failover_endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
        "sdk-direct live s09 failover connect failed",
        "sdk-direct live s09 failover boundary health check",
    )
}
