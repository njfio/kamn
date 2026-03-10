use super::{connect_agent, validate_non_empty};

pub(super) fn send_message_with_status(
    endpoint: &str,
    kolme_endpoint: &str,
    agent_name: &str,
    payload: &str,
    connect_context: &str,
    step: &str,
) -> Result<String, String> {
    let handle = connect_agent(endpoint, kolme_endpoint, agent_name, connect_context)?;
    let receipt = handle
        .send_message(payload)
        .map_err(|error| format!("{step} failed: {error}"))?;
    validate_s08_message_receipt_fields(
        receipt.message_id.as_str(),
        receipt.status.as_str(),
        step,
    )?;
    Ok(receipt.message_id)
}

pub(super) fn query_message_status(
    endpoint: &str,
    kolme_endpoint: &str,
    agent_name: &str,
    expected_message_id: &str,
    connect_context: &str,
    step: &str,
) -> Result<(), String> {
    let handle = connect_agent(endpoint, kolme_endpoint, agent_name, connect_context)?;
    let queried = handle
        .query_message(expected_message_id)
        .map_err(|error| format!("{step} failed: {error}"))?;
    validate_s08_query_message_response(
        expected_message_id,
        queried.message_id.as_str(),
        queried.status.as_str(),
        step,
    )
}

pub(super) fn require_health_status(
    endpoint: &str,
    kolme_endpoint: &str,
    agent_name: &str,
    connect_context: &str,
    step: &str,
) -> Result<(), String> {
    let handle = connect_agent(endpoint, kolme_endpoint, agent_name, connect_context)?;
    let health = handle
        .health()
        .map_err(|error| format!("{step} failed: {error}"))?;
    validate_non_empty(
        health.status.as_str(),
        &format!("{step} returned empty status"),
    )
}

pub(crate) fn validate_s08_message_receipt_fields(
    message_id: &str,
    status: &str,
    step: &str,
) -> Result<(), String> {
    validate_non_empty(message_id, &format!("{step} returned empty message_id"))?;
    validate_non_empty(status, &format!("{step} returned empty status"))
}

pub(crate) fn validate_s08_query_message_response(
    expected_message_id: &str,
    queried_message_id: &str,
    queried_status: &str,
    step: &str,
) -> Result<(), String> {
    if queried_message_id != expected_message_id {
        return Err(format!(
            "{step} returned mismatched message_id: expected={expected_message_id}, got={queried_message_id}"
        ));
    }
    validate_non_empty(queried_status, &format!("{step} returned empty status"))
}

pub(crate) fn validate_s08_distinct_message_ids(
    pre_message_id: &str,
    post_message_id: &str,
    step: &str,
) -> Result<(), String> {
    if post_message_id == pre_message_id {
        return Err(format!("{step} returned duplicate message_id"));
    }
    Ok(())
}
