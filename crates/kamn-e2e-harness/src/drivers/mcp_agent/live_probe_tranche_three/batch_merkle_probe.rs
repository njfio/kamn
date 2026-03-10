use super::super::*;
use std::env;

use crate::drivers::mcp_agent::live_probe_tranche_two::message_query_support::{
    query_message_with_validation, send_message_with_receipt, validate_distinct_message_ids,
};

pub(crate) fn run_live_s14_mcp_batch_merkle_probe() -> Result<(), String> {
    let settings = s14_settings()?;
    let batch_a_message_id = send_batch(&settings, "a", settings.batch_message_payload_a.as_str())?;
    let batch_b_message_id = send_batch(&settings, "b", settings.batch_message_payload_b.as_str())?;
    validate_distinct_message_ids(
        batch_a_message_id.as_str(),
        batch_b_message_id.as_str(),
        "mcp live s14 batch-b send_message",
    )?;
    query_batch(&settings, "a", batch_a_message_id.as_str())?;
    query_batch(&settings, "b", batch_b_message_id.as_str())?;
    let batch_root = batch_root(batch_a_message_id.as_str(), batch_b_message_id.as_str())?;
    verify_batch(
        &settings,
        "a",
        batch_a_message_id.as_str(),
        batch_root.as_str(),
    )?;
    verify_batch(
        &settings,
        "b",
        batch_b_message_id.as_str(),
        batch_root.as_str(),
    )
}

struct S14Settings {
    binary: String,
    endpoint: String,
    key_file: String,
    base_agent_name: String,
    batch_message_payload_a: String,
    batch_message_payload_b: String,
    block_height: u64,
    finality: String,
}

fn s14_settings() -> Result<S14Settings, String> {
    Ok(S14Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        base_agent_name: env_var_or_default("KAMN_E2E_S14_AGENT_NAME", DEFAULT_S14_AGENT_NAME),
        batch_message_payload_a: env::var("KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_A")
            .unwrap_or_else(|_| DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A.to_owned()),
        batch_message_payload_b: env::var("KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_B")
            .unwrap_or_else(|_| DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B.to_owned()),
        block_height: parse_block_height()?,
        finality: env_var_or_default("KAMN_E2E_S14_FINALITY", DEFAULT_S14_FINALITY),
    })
}

fn parse_block_height() -> Result<u64, String> {
    env::var("KAMN_E2E_S14_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("mcp live s14 invalid block height env value: {raw}"))
        })
        .transpose()
        .map(|value| value.unwrap_or(DEFAULT_S14_BLOCK_HEIGHT))
}

fn send_batch(settings: &S14Settings, label: &str, payload: &str) -> Result<String, String> {
    send_message_with_receipt(
        "mcp live s14",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-batch-{label}", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        format!("probe-send-message-batch-{label}").as_str(),
        payload,
        format!("mcp live s14 batch-{label} send_message").as_str(),
    )
}

fn query_batch(settings: &S14Settings, label: &str, message_id: &str) -> Result<(), String> {
    query_message_with_validation(
        "mcp live s14",
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-query-{label}", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        format!("probe-query-message-batch-{label}").as_str(),
        message_id,
        format!("mcp live s14 batch-{label} query_message").as_str(),
    )
}

fn batch_root(batch_a_message_id: &str, batch_b_message_id: &str) -> Result<String, String> {
    let batch_root = env::var("KAMN_E2E_S14_BATCH_ROOT")
        .unwrap_or_else(|_| format!("sha256:s14:{batch_a_message_id}:{batch_b_message_id}"));
    if !batch_root.trim().is_empty() {
        return Ok(batch_root);
    }
    Err("mcp live s14 batch-root marker must not be empty".to_owned())
}

fn verify_batch(
    settings: &S14Settings,
    label: &str,
    message_id: &str,
    batch_root: &str,
) -> Result<(), String> {
    let response = run_live_s14_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        format!("{}-proof-{label}", settings.base_agent_name).as_str(),
        settings.key_file.as_str(),
        format!("probe-verify-proof-batch-{label}").as_str(),
        "verify_proof",
        format!(
            "{{\"message_id\":\"{}\",\"tx_hash\":\"{}\",\"block_height\":\"{}\",\"finality\":\"{}\"}}",
            escape_json_scalar(message_id),
            escape_json_scalar(batch_root),
            settings.block_height,
            escape_json_scalar(settings.finality.as_str()),
        )
        .as_str(),
    )?;
    validate_s14_mcp_verify_proof_response(
        response.as_str(),
        message_id,
        format!("mcp live s14 batch-{label} verify_proof").as_str(),
    )
}
