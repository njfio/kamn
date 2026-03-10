use super::super::*;
use std::env;

pub(crate) fn run_live_s14_mcp_batch_merkle_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S14_AGENT_NAME", DEFAULT_S14_AGENT_NAME);
    let batch_message_payload_a = env::var("KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_A")
        .unwrap_or_else(|_| DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A.to_owned());
    let batch_message_payload_b = env::var("KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_B")
        .unwrap_or_else(|_| DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B.to_owned());
    let block_height = env::var("KAMN_E2E_S14_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("mcp live s14 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S14_BLOCK_HEIGHT);
    let finality = env_var_or_default("KAMN_E2E_S14_FINALITY", DEFAULT_S14_FINALITY);

    let batch_a_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(batch_message_payload_a.as_str())
    );
    let batch_a_send_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-batch-a").as_str(),
        key_file.as_str(),
        "probe-send-message-batch-a",
        "send_message",
        batch_a_send_arguments.as_str(),
    )?;
    let batch_a_message_id = validate_s08_mcp_message_receipt_fields(
        batch_a_send_response.as_str(),
        "mcp live s14 batch-a send_message",
    )?;

    let batch_b_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(batch_message_payload_b.as_str())
    );
    let batch_b_send_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-batch-b").as_str(),
        key_file.as_str(),
        "probe-send-message-batch-b",
        "send_message",
        batch_b_send_arguments.as_str(),
    )?;
    let batch_b_message_id = validate_s08_mcp_message_receipt_fields(
        batch_b_send_response.as_str(),
        "mcp live s14 batch-b send_message",
    )?;
    if batch_b_message_id == batch_a_message_id {
        return Err("mcp live s14 batch-b send_message returned duplicate message_id".to_owned());
    }

    let batch_a_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(batch_a_message_id.as_str())
    );
    let batch_a_query_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-query-a").as_str(),
        key_file.as_str(),
        "probe-query-message-batch-a",
        "query_message",
        batch_a_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        batch_a_query_response.as_str(),
        batch_a_message_id.as_str(),
        "mcp live s14 batch-a query_message",
    )?;

    let batch_b_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(batch_b_message_id.as_str())
    );
    let batch_b_query_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-query-b").as_str(),
        key_file.as_str(),
        "probe-query-message-batch-b",
        "query_message",
        batch_b_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        batch_b_query_response.as_str(),
        batch_b_message_id.as_str(),
        "mcp live s14 batch-b query_message",
    )?;

    let batch_root = env::var("KAMN_E2E_S14_BATCH_ROOT")
        .unwrap_or_else(|_| format!("sha256:s14:{}:{}", batch_a_message_id, batch_b_message_id));
    if batch_root.trim().is_empty() {
        return Err("mcp live s14 batch-root marker must not be empty".to_owned());
    }

    let batch_a_verify_arguments = format!(
        "{{\"message_id\":\"{}\",\"tx_hash\":\"{}\",\"block_height\":\"{}\",\"finality\":\"{}\"}}",
        escape_json_scalar(batch_a_message_id.as_str()),
        escape_json_scalar(batch_root.as_str()),
        block_height,
        escape_json_scalar(finality.as_str()),
    );
    let batch_a_verify_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-proof-a").as_str(),
        key_file.as_str(),
        "probe-verify-proof-batch-a",
        "verify_proof",
        batch_a_verify_arguments.as_str(),
    )?;
    validate_s14_mcp_verify_proof_response(
        batch_a_verify_response.as_str(),
        batch_a_message_id.as_str(),
        "mcp live s14 batch-a verify_proof",
    )?;

    let batch_b_verify_arguments = format!(
        "{{\"message_id\":\"{}\",\"tx_hash\":\"{}\",\"block_height\":\"{}\",\"finality\":\"{}\"}}",
        escape_json_scalar(batch_b_message_id.as_str()),
        escape_json_scalar(batch_root.as_str()),
        block_height,
        escape_json_scalar(finality.as_str()),
    );
    let batch_b_verify_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-proof-b").as_str(),
        key_file.as_str(),
        "probe-verify-proof-batch-b",
        "verify_proof",
        batch_b_verify_arguments.as_str(),
    )?;
    validate_s14_mcp_verify_proof_response(
        batch_b_verify_response.as_str(),
        batch_b_message_id.as_str(),
        "mcp live s14 batch-b verify_proof",
    )?;

    Ok(())
}
