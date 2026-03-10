use super::super::*;
use std::env;

use super::message_query_support::required_string_field;

pub(crate) fn run_live_s06_mcp_proof_verification_probe() -> Result<(), String> {
    let settings = s06_settings()?;
    let proof_response = verify_proof(&settings)?;
    validate_verified_proof(proof_response.as_str())
}

struct S06Settings {
    binary: String,
    endpoint: String,
    agent_name: String,
    key_file: String,
    proof_arguments: String,
}

fn s06_settings() -> Result<S06Settings, String> {
    let message_id = env::var("KAMN_E2E_S06_PROOF_MESSAGE_ID")
        .unwrap_or_else(|_| DEFAULT_S06_MESSAGE_ID.to_owned());
    let tx_hash = env_var_or_default("KAMN_E2E_S06_PROOF_TX_HASH", DEFAULT_S06_TX_HASH);
    let block_height = parse_block_height()?;
    let finality = env_var_or_default("KAMN_E2E_S06_PROOF_FINALITY", DEFAULT_S06_FINALITY);
    Ok(S06Settings {
        binary: env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY),
        endpoint: env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT),
        agent_name: env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME),
        key_file: env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE),
        proof_arguments: format!(
            "{{\"message_id\":\"{}\",\"tx_hash\":\"{}\",\"block_height\":\"{}\",\"finality\":\"{}\"}}",
            escape_json_scalar(message_id.as_str()),
            escape_json_scalar(tx_hash.as_str()),
            block_height,
            escape_json_scalar(finality.as_str()),
        ),
    })
}

fn parse_block_height() -> Result<u64, String> {
    env::var("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("mcp live s06 invalid block height env value: {raw}"))
        })
        .transpose()
        .map(|value| value.unwrap_or(DEFAULT_S06_BLOCK_HEIGHT))
}

fn verify_proof(settings: &S06Settings) -> Result<String, String> {
    run_live_s06_mcp_tool_call(
        settings.binary.as_str(),
        settings.endpoint.as_str(),
        settings.agent_name.as_str(),
        settings.key_file.as_str(),
        "probe-verify-proof",
        "verify_proof",
        settings.proof_arguments.as_str(),
    )
}

fn validate_verified_proof(proof_response: &str) -> Result<(), String> {
    if !proof_response.contains(r#""verified":true"#) {
        return Err(format!(
            "mcp live s06 verify_proof returned verified=false payload: {proof_response}"
        ));
    }
    validate_finality(proof_response)?;
    validate_block_height(proof_response)
}

fn validate_finality(proof_response: &str) -> Result<(), String> {
    let proof_finality =
        required_string_field(proof_response, "finality", "mcp live s06 verify_proof")?;
    if proof_finality.trim() == "FINAL" {
        return Ok(());
    }
    Err(format!(
        "mcp live s06 verify_proof returned non-final finality: {proof_finality}"
    ))
}

fn validate_block_height(proof_response: &str) -> Result<(), String> {
    let proof_block_height =
        json_optional_u64_field(proof_response, "block_height").ok_or_else(|| {
            format!(
                "mcp live s06 verify_proof response missing block_height field: {proof_response}"
            )
        })?;
    if proof_block_height != 0 {
        return Ok(());
    }
    Err("mcp live s06 verify_proof returned block_height=0".to_owned())
}
