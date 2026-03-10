use std::env;
use std::time::Instant;

use super::{
    default_endpoint, kolme_endpoint, DEFAULT_S15_AGENT_NAME, DEFAULT_S15_ITERATIONS,
    DEFAULT_S15_MAX_P50_MILLIS, DEFAULT_S15_MAX_P99_MILLIS, DEFAULT_S15_MAX_TOTAL_MILLIS,
    DEFAULT_S15_MESSAGE_PAYLOAD,
};

pub(super) fn run_live_s15_performance_smoke_probe() -> Result<(), String> {
    let settings = s15_settings()?;
    let (send_handle, query_handle) = connect_s15_handles(&settings)?;
    let (latency_samples, total_elapsed_millis) =
        collect_latency_samples(&settings, &send_handle, &query_handle)?;
    super::super::validate_s15_latency_budget_samples(
        latency_samples.as_slice(),
        total_elapsed_millis,
        settings.max_total_millis,
        settings.max_p50_millis,
        settings.max_p99_millis,
        "sdk-direct live s15 performance-smoke",
    )
}

struct S15Settings {
    endpoint: String,
    kolme_endpoint: String,
    base_agent_name: String,
    message_payload: String,
    iterations: u64,
    max_total_millis: u128,
    max_p50_millis: u128,
    max_p99_millis: u128,
}

fn s15_settings() -> Result<S15Settings, String> {
    let iterations = parse_s15_iterations()?;
    if iterations == 0 {
        return Err("sdk-direct live s15 iterations must be greater than zero".to_owned());
    }
    Ok(S15Settings {
        endpoint: default_endpoint(),
        kolme_endpoint: kolme_endpoint(),
        base_agent_name: s15_agent_name(),
        message_payload: s15_message_payload(),
        iterations,
        max_total_millis: s15_total_budget()?,
        max_p50_millis: s15_p50_budget()?,
        max_p99_millis: s15_p99_budget()?,
    })
}

fn parse_s15_iterations() -> Result<u64, String> {
    env::var("KAMN_E2E_S15_ITERATIONS")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("sdk-direct live s15 invalid iterations env value: {raw}"))
        })
        .transpose()
        .map(|value| value.unwrap_or(DEFAULT_S15_ITERATIONS))
}

fn parse_budget(name: &str, default_value: u128, context: &str) -> Result<u128, String> {
    super::super::parse_s15_budget_env_u128(name, default_value, context)
}

fn s15_agent_name() -> String {
    super::super::env_var_or_default("KAMN_E2E_S15_AGENT_NAME", DEFAULT_S15_AGENT_NAME)
}

fn s15_message_payload() -> String {
    super::super::env_var_or_default("KAMN_E2E_S15_MESSAGE_PAYLOAD", DEFAULT_S15_MESSAGE_PAYLOAD)
}

fn s15_total_budget() -> Result<u128, String> {
    parse_budget(
        "KAMN_E2E_S15_MAX_TOTAL_MILLIS",
        DEFAULT_S15_MAX_TOTAL_MILLIS,
        "sdk-direct live s15 max-total budget",
    )
}

fn s15_p50_budget() -> Result<u128, String> {
    parse_budget(
        "KAMN_E2E_S15_MAX_P50_MILLIS",
        DEFAULT_S15_MAX_P50_MILLIS,
        "sdk-direct live s15 max-p50 budget",
    )
}

fn s15_p99_budget() -> Result<u128, String> {
    parse_budget(
        "KAMN_E2E_S15_MAX_P99_MILLIS",
        DEFAULT_S15_MAX_P99_MILLIS,
        "sdk-direct live s15 max-p99 budget",
    )
}

fn connect_s15_handles(
    settings: &S15Settings,
) -> Result<(super::KamnAgentHandle, super::KamnAgentHandle), String> {
    let send_handle = super::connect_agent(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-send", settings.base_agent_name).as_str(),
        "sdk-direct live s15 send connect failed",
    )?;
    let query_handle = super::connect_agent(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-query", settings.base_agent_name).as_str(),
        "sdk-direct live s15 query connect failed",
    )?;
    Ok((send_handle, query_handle))
}

fn collect_latency_samples(
    settings: &S15Settings,
    send_handle: &super::KamnAgentHandle,
    query_handle: &super::KamnAgentHandle,
) -> Result<(Vec<u128>, u128), String> {
    let total_start = Instant::now();
    let mut latency_samples = Vec::with_capacity(settings.iterations as usize);
    for iteration in 0..settings.iterations {
        latency_samples.push(run_s15_iteration(
            send_handle,
            query_handle,
            settings,
            iteration,
        )?);
    }
    Ok((latency_samples, total_start.elapsed().as_millis()))
}

fn run_s15_iteration(
    send_handle: &super::KamnAgentHandle,
    query_handle: &super::KamnAgentHandle,
    settings: &S15Settings,
    iteration: u64,
) -> Result<u128, String> {
    let iteration_start = Instant::now();
    let message_id = send_iteration_message(send_handle, settings, iteration)?;
    query_iteration_message(query_handle, message_id.as_str(), iteration)?;
    Ok(iteration_start.elapsed().as_millis())
}

fn send_iteration_message(
    send_handle: &super::KamnAgentHandle,
    settings: &S15Settings,
    iteration: u64,
) -> Result<String, String> {
    let send_receipt = send_handle
        .send_message(settings.message_payload.as_str())
        .map_err(|error| {
            format!("sdk-direct live s15 send-message failed at iteration {iteration}: {error}")
        })?;
    super::super::validate_s08_message_receipt_fields(
        send_receipt.message_id.as_str(),
        send_receipt.status.as_str(),
        "sdk-direct live s15 send-message",
    )?;
    Ok(send_receipt.message_id)
}

fn query_iteration_message(
    query_handle: &super::KamnAgentHandle,
    message_id: &str,
    iteration: u64,
) -> Result<(), String> {
    let queried_status = query_handle.query_message(message_id).map_err(|error| {
        format!("sdk-direct live s15 query-message failed at iteration {iteration}: {error}")
    })?;
    super::super::validate_s08_query_message_response(
        message_id,
        queried_status.message_id.as_str(),
        queried_status.status.as_str(),
        "sdk-direct live s15 query-message",
    )
}
