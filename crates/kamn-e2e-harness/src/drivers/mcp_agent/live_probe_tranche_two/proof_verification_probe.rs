use super::super::*;
use std::env;

pub(crate) fn run_live_s06_mcp_proof_verification_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let message_id = env::var("KAMN_E2E_S06_PROOF_MESSAGE_ID")
        .unwrap_or_else(|_| DEFAULT_S06_MESSAGE_ID.to_owned());
    let tx_hash = env_var_or_default("KAMN_E2E_S06_PROOF_TX_HASH", DEFAULT_S06_TX_HASH);
    let block_height = env::var("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("mcp live s06 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S06_BLOCK_HEIGHT);
    let finality = env_var_or_default("KAMN_E2E_S06_PROOF_FINALITY", DEFAULT_S06_FINALITY);

    let proof_arguments = format!(
        "{{\"message_id\":\"{}\",\"tx_hash\":\"{}\",\"block_height\":\"{}\",\"finality\":\"{}\"}}",
        escape_json_scalar(message_id.as_str()),
        escape_json_scalar(tx_hash.as_str()),
        block_height,
        escape_json_scalar(finality.as_str()),
    );
    let proof_response = run_live_s06_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        agent_name.as_str(),
        key_file.as_str(),
        "probe-verify-proof",
        "verify_proof",
        proof_arguments.as_str(),
    )?;

    if !proof_response.contains(r#""verified":true"#) {
        return Err(format!(
            "mcp live s06 verify_proof returned verified=false payload: {proof_response}"
        ));
    }
    let proof_finality = json_optional_string_field(proof_response.as_str(), "finality")
        .ok_or_else(|| {
            format!("mcp live s06 verify_proof response missing finality field: {proof_response}")
        })?;
    if proof_finality.trim() != "FINAL" {
        return Err(format!(
            "mcp live s06 verify_proof returned non-final finality: {proof_finality}"
        ));
    }
    let proof_block_height = json_optional_u64_field(proof_response.as_str(), "block_height")
        .ok_or_else(|| {
            format!(
                "mcp live s06 verify_proof response missing block_height field: {proof_response}"
            )
        })?;
    if proof_block_height == 0 {
        return Err("mcp live s06 verify_proof returned block_height=0".to_owned());
    }

    Ok(())
}
