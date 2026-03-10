use super::super::*;
use std::env;

use crate::drivers::mcp_agent::live_probe_tranche_two::message_query_support::{
    query_message_with_validation, send_message_with_receipt,
};

pub(crate) fn run_live_s15_mcp_performance_smoke_probe() -> Result<(), String> {
    let settings = s15_settings()?;
    let (latency_samples, total_elapsed_millis) = collect_latency_samples(&settings)?;
    validate_s15_latency_budget_samples(
        latency_samples.as_slice(),
        total_elapsed_millis,
        settings.max_total_millis,
        settings.max_p50_millis,
        settings.max_p99_millis,
        "mcp live s15 performance-smoke",
    )
}

struct S15Settings {
    binary: String,
    endpoint: String,
    base_agent_name: String,
    key_file: String,
    message_payload: String,
    iterations: u64,
    max_total_millis: u128,
    max_p50_millis: u128,
    max_p99_millis: u128,
}

fn s15_settings() -> Result<S15Settings, String> {
    Ok(S15Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        base_agent_name: env_var_or_default("KAMN_E2E_S15_AGENT_NAME", DEFAULT_S15_AGENT_NAME),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        message_payload: env::var("KAMN_E2E_S15_MESSAGE_PAYLOAD")
            .unwrap_or_else(|_| DEFAULT_S15_MESSAGE_PAYLOAD.to_owned()),
        iterations: parse_iterations()?,
        max_total_millis: parse_s15_budget_env_u128(
            "KAMN_E2E_S15_MAX_TOTAL_MILLIS",
            DEFAULT_S15_MAX_TOTAL_MILLIS,
            "mcp live s15 max-total budget",
        )?,
        max_p50_millis: parse_s15_budget_env_u128(
            "KAMN_E2E_S15_MAX_P50_MILLIS",
            DEFAULT_S15_MAX_P50_MILLIS,
            "mcp live s15 max-p50 budget",
        )?,
        max_p99_millis: parse_s15_budget_env_u128(
            "KAMN_E2E_S15_MAX_P99_MILLIS",
            DEFAULT_S15_MAX_P99_MILLIS,
            "mcp live s15 max-p99 budget",
        )?,
    })
}

fn parse_iterations() -> Result<u64, String> {
    let iterations = env::var("KAMN_E2E_S15_ITERATIONS")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("mcp live s15 invalid iterations env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S15_ITERATIONS);
    if iterations != 0 {
        return Ok(iterations);
    }
    Err("mcp live s15 iterations must be greater than zero".to_owned())
}

fn collect_latency_samples(settings: &S15Settings) -> Result<(Vec<u128>, u128), String> {
    let total_start = std::time::Instant::now();
    let mut latency_samples = Vec::with_capacity(settings.iterations as usize);
    for iteration in 0..settings.iterations {
        latency_samples.push(run_iteration(settings, iteration)?);
    }
    Ok((latency_samples, total_start.elapsed().as_millis()))
}

fn run_iteration(settings: &S15Settings, iteration: u64) -> Result<u128, String> {
    let iteration_start = std::time::Instant::now();
    let message_id = send_message_with_receipt(
        "mcp live s15",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-send-{iteration}", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        &format!("probe-send-message-s15-{iteration}"),
        settings.message_payload.as_str(),
        "mcp live s15 send_message",
    )?;
    query_message_with_validation(
        "mcp live s15",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-query-{iteration}", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        &format!("probe-query-message-s15-{iteration}"),
        message_id.as_str(),
        "mcp live s15 query_message",
    )?;
    Ok(iteration_start.elapsed().as_millis())
}
