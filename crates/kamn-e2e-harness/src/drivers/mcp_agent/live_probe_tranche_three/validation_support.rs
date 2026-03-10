use super::super::*;

pub(crate) fn validate_s14_mcp_verify_proof_response(
    response: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    validate_message_id(response, expected_message_id, step)?;
    validate_verified(response, step)?;
    validate_finality(response, step)?;
    validate_block_height(response, step)
}

fn validate_message_id(
    response: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let observed = json_optional_string_field(response, "message_id")
        .ok_or_else(|| format!("{step} response missing message_id field: {response}"))?;
    if observed == expected_message_id {
        return Ok(());
    }
    Err(format!(
        "{step} returned mismatched message_id: expected={expected_message_id}, got={observed}"
    ))
}

fn validate_verified(response: &str, step: &str) -> Result<(), String> {
    if response.contains(r#""verified":true"#) {
        return Ok(());
    }
    Err(format!(
        "{step} returned verified=false payload: {response}"
    ))
}

fn validate_finality(response: &str, step: &str) -> Result<(), String> {
    let observed = json_optional_string_field(response, "finality")
        .ok_or_else(|| format!("{step} response missing finality field: {response}"))?;
    if observed == "FINAL" {
        return Ok(());
    }
    Err(format!("{step} returned non-final finality: {observed}"))
}

fn validate_block_height(response: &str, step: &str) -> Result<(), String> {
    let observed = json_optional_u64_field(response, "block_height")
        .ok_or_else(|| format!("{step} response missing block_height field: {response}"))?;
    if observed != 0 {
        return Ok(());
    }
    Err(format!("{step} returned block_height=0"))
}
