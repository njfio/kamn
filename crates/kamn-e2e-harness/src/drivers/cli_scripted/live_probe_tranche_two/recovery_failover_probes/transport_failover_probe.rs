use super::{s09_settings, S09Settings};
use crate::drivers::cli_scripted::live_probe_tranche_two::message_query_support::{
    query_message_status, require_health_status, send_message_with_status,
};
use crate::drivers::cli_scripted::live_probe_tranche_two::validate_s08_distinct_message_ids;

pub(super) fn run_live_s09_cli_transport_failover_probe() -> Result<(), String> {
    let settings = s09_settings();
    let pre_message_id = send_pre_message(&settings)?;
    query_pre_message(&settings, pre_message_id.as_str())?;
    require_boundary_health(&settings)?;
    let post_message_id = send_post_message(&settings)?;
    validate_s08_distinct_message_ids(
        pre_message_id.as_str(),
        post_message_id.as_str(),
        "cli live s09 post-failover send-message",
    )?;
    query_post_message(&settings, post_message_id.as_str())
}

fn send_pre_message(settings: &S09Settings) -> Result<String, String> {
    send_message_with_status(
        settings.primary_endpoint.as_str(),
        format!("{}-pre-send", settings.base_agent_name).as_str(),
        settings.pre_message_payload.as_str(),
        "cli live s09 pre-failover send-message",
    )
}

fn query_pre_message(settings: &S09Settings, message_id: &str) -> Result<(), String> {
    query_message_status(
        settings.primary_endpoint.as_str(),
        format!("{}-pre-query", settings.base_agent_name).as_str(),
        message_id,
        "cli live s09 pre-failover query-message",
    )
}

fn require_boundary_health(settings: &S09Settings) -> Result<(), String> {
    require_health_status(
        settings.failover_endpoint.as_str(),
        format!("{}-boundary", settings.base_agent_name).as_str(),
        "cli live s09 failover boundary health check",
    )
}

fn send_post_message(settings: &S09Settings) -> Result<String, String> {
    send_message_with_status(
        settings.failover_endpoint.as_str(),
        format!("{}-post-send", settings.base_agent_name).as_str(),
        settings.post_message_payload.as_str(),
        "cli live s09 post-failover send-message",
    )
}

fn query_post_message(settings: &S09Settings, message_id: &str) -> Result<(), String> {
    query_message_status(
        settings.failover_endpoint.as_str(),
        format!("{}-post-query", settings.base_agent_name).as_str(),
        message_id,
        "cli live s09 post-failover query-message",
    )
}
