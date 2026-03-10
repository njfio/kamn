use super::super::*;

pub(crate) fn validate_s08_mcp_message_receipt_fields(
    response: &str,
    step: &str,
) -> Result<String, String> {
    let message_id = json_optional_string_field(response, "message_id")
        .ok_or_else(|| format!("{step} response missing message_id field: {response}"))?;
    if message_id.trim().is_empty() {
        return Err(format!("{step} returned empty message_id"));
    }
    let status = json_optional_string_field(response, "status")
        .ok_or_else(|| format!("{step} response missing status field: {response}"))?;
    if status.trim().is_empty() {
        return Err(format!("{step} returned empty status"));
    }
    Ok(message_id)
}

pub(crate) fn validate_s08_mcp_query_message_response(
    response: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let queried_message_id = json_optional_string_field(response, "message_id")
        .ok_or_else(|| format!("{step} response missing message_id field: {response}"))?;
    if queried_message_id != expected_message_id {
        return Err(format!(
            "{step} returned mismatched message_id: expected={expected_message_id}, got={queried_message_id}"
        ));
    }
    let queried_status = json_optional_string_field(response, "status")
        .ok_or_else(|| format!("{step} response missing status field: {response}"))?;
    if queried_status.trim().is_empty() {
        return Err(format!("{step} returned empty status"));
    }
    Ok(())
}
