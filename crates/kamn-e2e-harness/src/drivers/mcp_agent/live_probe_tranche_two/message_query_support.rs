use super::super::*;

pub(crate) fn validate_s08_mcp_message_receipt_fields(
    response: &str,
    step: &str,
) -> Result<String, String> {
    let message_id = required_string_field(response, "message_id", step)?;
    require_non_empty(message_id.as_str(), step, "message_id")?;
    let status = required_string_field(response, "status", step)?;
    require_non_empty(status.as_str(), step, "status")?;
    Ok(message_id)
}

pub(crate) fn validate_s08_mcp_query_message_response(
    response: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let queried_message_id = required_string_field(response, "message_id", step)?;
    validate_message_id_match(queried_message_id.as_str(), expected_message_id, step)?;
    let queried_status = required_string_field(response, "status", step)?;
    require_non_empty(queried_status.as_str(), step, "status")
}

pub(crate) fn send_message_with_receipt(
    step_prefix: &str,
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    payload: &str,
    step: &str,
) -> Result<String, String> {
    let response = run_named_mcp_tool_call(
        step_prefix,
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        "send_message",
        payload_arguments(payload).as_str(),
    )?;
    validate_s08_mcp_message_receipt_fields(response.as_str(), step)
}

pub(crate) fn query_message_with_validation(
    step_prefix: &str,
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    message_id: &str,
    step: &str,
) -> Result<(), String> {
    let response = run_named_mcp_tool_call(
        step_prefix,
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        "query_message",
        message_id_arguments(message_id).as_str(),
    )?;
    validate_s08_mcp_query_message_response(response.as_str(), message_id, step)
}

pub(crate) fn health_status(
    step_prefix: &str,
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    step: &str,
) -> Result<String, String> {
    let response = run_named_mcp_tool_call(
        step_prefix,
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        "health",
        "{}",
    )?;
    let status = required_string_field(response.as_str(), "status", step)?;
    require_non_empty(status.as_str(), step, "status")?;
    Ok(status)
}

pub(crate) fn payload_arguments(payload: &str) -> String {
    format!("{{\"payload\":\"{}\"}}", escape_json_scalar(payload))
}

pub(crate) fn message_id_arguments(message_id: &str) -> String {
    format!("{{\"message_id\":\"{}\"}}", escape_json_scalar(message_id))
}

pub(crate) fn validate_distinct_message_ids(
    left: &str,
    right: &str,
    step: &str,
) -> Result<(), String> {
    if left != right {
        return Ok(());
    }
    Err(format!("{step} returned duplicate message_id"))
}

pub(crate) fn required_string_field(
    response: &str,
    key: &str,
    step: &str,
) -> Result<String, String> {
    json_optional_string_field(response, key)
        .ok_or_else(|| format!("{step} response missing {key} field: {response}"))
}

pub(crate) fn require_non_empty(value: &str, step: &str, key: &str) -> Result<(), String> {
    if !value.trim().is_empty() {
        return Ok(());
    }
    Err(format!("{step} returned empty {key}"))
}

fn validate_message_id_match(
    observed: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    if observed == expected_message_id {
        return Ok(());
    }
    Err(format!(
        "{step} returned mismatched message_id: expected={expected_message_id}, got={observed}"
    ))
}
