use super::super::*;

pub(crate) fn validate_s14_mcp_verify_proof_response(
    response: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let observed_message_id = json_optional_string_field(response, "message_id")
        .ok_or_else(|| format!("{step} response missing message_id field: {response}"))?;
    if observed_message_id != expected_message_id {
        return Err(format!(
            "{step} returned mismatched message_id: expected={expected_message_id}, got={observed_message_id}"
        ));
    }
    if !response.contains(r#""verified":true"#) {
        return Err(format!(
            "{step} returned verified=false payload: {response}"
        ));
    }
    let observed_finality = json_optional_string_field(response, "finality")
        .ok_or_else(|| format!("{step} response missing finality field: {response}"))?;
    if observed_finality != "FINAL" {
        return Err(format!(
            "{step} returned non-final finality: {observed_finality}"
        ));
    }
    let observed_block_height = json_optional_u64_field(response, "block_height")
        .ok_or_else(|| format!("{step} response missing block_height field: {response}"))?;
    if observed_block_height == 0 {
        return Err(format!("{step} returned block_height=0"));
    }
    Ok(())
}
