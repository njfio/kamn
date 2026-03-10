use std::env;
use std::time::Instant;

use super::{
    default_endpoint, env_payload, env_value, DEFAULT_S15_AGENT_NAME, DEFAULT_S15_ITERATIONS,
    DEFAULT_S15_MAX_P50_MILLIS, DEFAULT_S15_MAX_P99_MILLIS, DEFAULT_S15_MAX_TOTAL_MILLIS,
    DEFAULT_S15_MESSAGE_PAYLOAD,
};

pub(super) fn run_live_s15_cli_performance_smoke_probe() -> Result<(), String> {
    let settings = s15_settings()?;
    let (latency_samples, total_elapsed_millis) = collect_latency_samples(&settings)?;
    super::super::validate_s15_latency_budget_samples(
        latency_samples.as_slice(),
        total_elapsed_millis,
        settings.max_total_millis,
        settings.max_p50_millis,
        settings.max_p99_millis,
        "cli live s15 performance-smoke",
    )
}

struct S15Settings {
    endpoint: String,
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
        return Err("cli live s15 iterations must be greater than zero".to_owned());
    }
    let (max_total_millis, max_p50_millis, max_p99_millis) = s15_budgets()?;
    Ok(S15Settings {
        endpoint: default_endpoint(),
        base_agent_name: env_value("KAMN_E2E_S15_AGENT_NAME", DEFAULT_S15_AGENT_NAME),
        message_payload: env_payload("KAMN_E2E_S15_MESSAGE_PAYLOAD", DEFAULT_S15_MESSAGE_PAYLOAD),
        iterations,
        max_total_millis,
        max_p50_millis,
        max_p99_millis,
    })
}

fn s15_budgets() -> Result<(u128, u128, u128), String> {
    Ok((
        parse_budget(
            "KAMN_E2E_S15_MAX_TOTAL_MILLIS",
            DEFAULT_S15_MAX_TOTAL_MILLIS,
            "cli live s15 max-total budget",
        )?,
        parse_budget(
            "KAMN_E2E_S15_MAX_P50_MILLIS",
            DEFAULT_S15_MAX_P50_MILLIS,
            "cli live s15 max-p50 budget",
        )?,
        parse_budget(
            "KAMN_E2E_S15_MAX_P99_MILLIS",
            DEFAULT_S15_MAX_P99_MILLIS,
            "cli live s15 max-p99 budget",
        )?,
    ))
}

fn parse_s15_iterations() -> Result<u64, String> {
    env::var("KAMN_E2E_S15_ITERATIONS")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("cli live s15 invalid iterations env value: {raw}"))
        })
        .transpose()
        .map(|value| value.unwrap_or(DEFAULT_S15_ITERATIONS))
}

fn parse_budget(name: &str, default_value: u128, context: &str) -> Result<u128, String> {
    super::super::parse_s15_budget_env_u128(name, default_value, context)
}

fn collect_latency_samples(settings: &S15Settings) -> Result<(Vec<u128>, u128), String> {
    let total_start = Instant::now();
    let mut latency_samples = Vec::with_capacity(settings.iterations as usize);
    for iteration in 0..settings.iterations {
        latency_samples.push(run_s15_iteration(settings, iteration)?);
    }
    Ok((latency_samples, total_start.elapsed().as_millis()))
}

fn run_s15_iteration(settings: &S15Settings, iteration: u64) -> Result<u128, String> {
    let iteration_start = Instant::now();
    let message_id = send_iteration_message(settings, iteration)?;
    query_iteration_message(settings, message_id.as_str(), iteration)?;
    Ok(iteration_start.elapsed().as_millis())
}

fn send_iteration_message(settings: &S15Settings, iteration: u64) -> Result<String, String> {
    let send_output = super::super::run_cli_command_capture_stdout_with_agent_name(
        super::cli_binary().as_str(),
        &[
            "send-message",
            "--endpoint",
            settings.endpoint.as_str(),
            "--format",
            "text",
            settings.message_payload.as_str(),
        ],
        "cli live s15 send-message",
        format!("{}-send-{iteration}", settings.base_agent_name).as_str(),
    )?;
    super::super::validate_s08_message_receipt_fields(
        send_output.as_str(),
        "cli live s15 send-message",
    )
}

fn query_iteration_message(
    settings: &S15Settings,
    message_id: &str,
    iteration: u64,
) -> Result<(), String> {
    let query_output = super::super::run_cli_command_capture_stdout_with_agent_name(
        super::cli_binary().as_str(),
        &[
            "query-message",
            "--endpoint",
            settings.endpoint.as_str(),
            "--format",
            "text",
            message_id,
        ],
        "cli live s15 query-message",
        format!("{}-query-{iteration}", settings.base_agent_name).as_str(),
    )?;
    super::super::validate_s08_query_message_response(
        query_output.as_str(),
        message_id,
        "cli live s15 query-message",
    )
}
