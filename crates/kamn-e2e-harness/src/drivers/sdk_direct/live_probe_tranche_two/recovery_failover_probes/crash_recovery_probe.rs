use super::{s08_settings, S08Settings};
use crate::drivers::sdk_direct::live_probe_tranche_two::message_query_support::{
    query_message_status, require_health_status, send_message_with_status,
};
use crate::drivers::sdk_direct::live_probe_tranche_two::validate_s08_distinct_message_ids;

pub(super) fn run_live_s08_crash_recovery_probe() -> Result<(), String> {
    let settings = s08_settings();
    let pre_message_id = send_s08_message(
        &settings,
        "pre-send",
        settings.pre_message_payload.as_str(),
        "pre-boundary",
    )?;
    query_s08_message(
        &settings,
        "pre-query",
        pre_message_id.as_str(),
        "pre-boundary",
    )?;
    require_s08_health(&settings, "boundary")?;
    let post_message_id = send_s08_message(
        &settings,
        "post-send",
        settings.post_message_payload.as_str(),
        "post-boundary",
    )?;
    validate_s08_distinct_message_ids(
        pre_message_id.as_str(),
        post_message_id.as_str(),
        "sdk-direct live s08 post-boundary send-message",
    )?;
    query_s08_message(
        &settings,
        "post-query",
        post_message_id.as_str(),
        "post-boundary",
    )
}

fn send_s08_message(
    settings: &S08Settings,
    suffix: &str,
    payload: &str,
    phase: &str,
) -> Result<String, String> {
    send_message_with_status(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
        payload,
        "sdk-direct live s08 connect failed",
        &format!("sdk-direct live s08 {phase} send-message"),
    )
}

fn query_s08_message(
    settings: &S08Settings,
    suffix: &str,
    message_id: &str,
    phase: &str,
) -> Result<(), String> {
    query_message_status(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
        message_id,
        "sdk-direct live s08 connect failed",
        &format!("sdk-direct live s08 {phase} query-message"),
    )
}

fn require_s08_health(settings: &S08Settings, suffix: &str) -> Result<(), String> {
    require_health_status(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
        "sdk-direct live s08 connect failed",
        "sdk-direct live s08 boundary health check",
    )
}
