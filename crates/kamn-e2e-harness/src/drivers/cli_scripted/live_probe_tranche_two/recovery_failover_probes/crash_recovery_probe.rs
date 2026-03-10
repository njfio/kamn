use super::{s08_settings, S08Settings};
use crate::drivers::cli_scripted::live_probe_tranche_two::message_query_support::{
    query_message_status, require_health_status, send_message_with_status,
};
use crate::drivers::cli_scripted::live_probe_tranche_two::validate_s08_distinct_message_ids;

pub(super) fn run_live_s08_cli_crash_recovery_probe() -> Result<(), String> {
    let settings = s08_settings();
    let pre_message_id = send_pre_message(&settings)?;
    query_pre_message(&settings, pre_message_id.as_str())?;
    require_boundary_health(&settings)?;
    let post_message_id = send_post_message(&settings)?;
    validate_s08_distinct_message_ids(
        pre_message_id.as_str(),
        post_message_id.as_str(),
        "cli live s08 post-boundary send-message",
    )?;
    query_post_message(&settings, post_message_id.as_str())
}

fn send_pre_message(settings: &S08Settings) -> Result<String, String> {
    send_message_with_status(
        settings.endpoint.as_str(),
        format!("{}-pre-send", settings.base_agent_name).as_str(),
        settings.pre_message_payload.as_str(),
        "cli live s08 pre-boundary send-message",
    )
}

fn query_pre_message(settings: &S08Settings, message_id: &str) -> Result<(), String> {
    query_message_status(
        settings.endpoint.as_str(),
        format!("{}-pre-query", settings.base_agent_name).as_str(),
        message_id,
        "cli live s08 pre-boundary query-message",
    )
}

fn require_boundary_health(settings: &S08Settings) -> Result<(), String> {
    require_health_status(
        settings.endpoint.as_str(),
        format!("{}-boundary", settings.base_agent_name).as_str(),
        "cli live s08 boundary health check",
    )
}

fn send_post_message(settings: &S08Settings) -> Result<String, String> {
    send_message_with_status(
        settings.endpoint.as_str(),
        format!("{}-post-send", settings.base_agent_name).as_str(),
        settings.post_message_payload.as_str(),
        "cli live s08 post-boundary send-message",
    )
}

fn query_post_message(settings: &S08Settings, message_id: &str) -> Result<(), String> {
    query_message_status(
        settings.endpoint.as_str(),
        format!("{}-post-query", settings.base_agent_name).as_str(),
        message_id,
        "cli live s08 post-boundary query-message",
    )
}
