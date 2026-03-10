use super::super::*;
use std::env;

pub(crate) fn run_live_s15_mcp_performance_smoke_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let base_agent_name = env_var_or_default("KAMN_E2E_S15_AGENT_NAME", DEFAULT_S15_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let message_payload = env::var("KAMN_E2E_S15_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S15_MESSAGE_PAYLOAD.to_owned());
    let iterations = env::var("KAMN_E2E_S15_ITERATIONS")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("mcp live s15 invalid iterations env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S15_ITERATIONS);
    if iterations == 0 {
        return Err("mcp live s15 iterations must be greater than zero".to_owned());
    }

    let max_total_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_TOTAL_MILLIS",
        DEFAULT_S15_MAX_TOTAL_MILLIS,
        "mcp live s15 max-total budget",
    )?;
    let max_p50_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_P50_MILLIS",
        DEFAULT_S15_MAX_P50_MILLIS,
        "mcp live s15 max-p50 budget",
    )?;
    let max_p99_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_P99_MILLIS",
        DEFAULT_S15_MAX_P99_MILLIS,
        "mcp live s15 max-p99 budget",
    )?;

    let send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let total_start = std::time::Instant::now();
    let mut latency_samples = Vec::with_capacity(iterations as usize);
    for iteration in 0..iterations {
        let iteration_start = std::time::Instant::now();
        let send_response = run_live_s15_mcp_tool_call(
            binary.as_str(),
            endpoint.as_str(),
            format!("{base_agent_name}-send-{iteration}").as_str(),
            key_file.as_str(),
            format!("probe-send-message-s15-{iteration}").as_str(),
            "send_message",
            send_arguments.as_str(),
        )?;
        let message_id = validate_s08_mcp_message_receipt_fields(
            send_response.as_str(),
            "mcp live s15 send_message",
        )?;

        let query_arguments = format!(
            "{{\"message_id\":\"{}\"}}",
            escape_json_scalar(message_id.as_str())
        );
        let query_response = run_live_s15_mcp_tool_call(
            binary.as_str(),
            endpoint.as_str(),
            format!("{base_agent_name}-query-{iteration}").as_str(),
            key_file.as_str(),
            format!("probe-query-message-s15-{iteration}").as_str(),
            "query_message",
            query_arguments.as_str(),
        )?;
        validate_s08_mcp_query_message_response(
            query_response.as_str(),
            message_id.as_str(),
            "mcp live s15 query_message",
        )?;

        latency_samples.push(iteration_start.elapsed().as_millis());
    }
    let total_elapsed_millis = total_start.elapsed().as_millis();

    validate_s15_latency_budget_samples(
        latency_samples.as_slice(),
        total_elapsed_millis,
        max_total_millis,
        max_p50_millis,
        max_p99_millis,
        "mcp live s15 performance-smoke",
    )
}
