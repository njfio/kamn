use super::{connect_agent, validate_non_empty};

pub(super) fn send_message_with_validated_receipt(
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
    super::super::validate_s08_message_receipt_fields(
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
    super::super::validate_s08_query_message_response(
        expected_message_id,
        queried.message_id.as_str(),
        queried.status.as_str(),
        step,
    )
}

pub(crate) fn validate_s14_proof_response(
    expected_message_id: &str,
    observed_message_id: &str,
    observed_block_height: u64,
    observed_finality: &str,
    observed_verified: bool,
    step: &str,
) -> Result<(), String> {
    validate_s14_message_id(expected_message_id, observed_message_id, step)?;
    validate_s14_verified(observed_verified, step)?;
    validate_s14_finality(observed_finality, step)?;
    validate_s14_block_height(observed_block_height, step)
}

pub(super) fn validate_content_state(
    lifecycle_state: &str,
    redaction_status: &str,
    step: &str,
) -> Result<(), String> {
    validate_non_empty(
        lifecycle_state,
        &format!("{step} returned empty lifecycle_state"),
    )?;
    validate_non_empty(
        redaction_status,
        &format!("{step} returned empty redaction_status"),
    )
}

pub(super) fn validate_bridge_forward_fields(
    bridge_status: &str,
    target_message_id: &str,
    forward_tx_hash: &str,
    step: &str,
) -> Result<(), String> {
    validate_non_empty(
        bridge_status,
        &format!("{step} returned empty bridge_status"),
    )?;
    validate_non_empty(
        target_message_id,
        &format!("{step} returned empty target_message_id"),
    )?;
    validate_non_empty(
        forward_tx_hash,
        &format!("{step} returned empty forward_tx_hash"),
    )
}

fn validate_s14_message_id(
    expected_message_id: &str,
    observed_message_id: &str,
    step: &str,
) -> Result<(), String> {
    if observed_message_id == expected_message_id {
        return Ok(());
    }
    Err(format!(
        "{step} returned mismatched message_id: expected={expected_message_id}, got={observed_message_id}"
    ))
}

fn validate_s14_verified(observed_verified: bool, step: &str) -> Result<(), String> {
    if observed_verified {
        return Ok(());
    }
    Err(format!("{step} returned verified=false"))
}

fn validate_s14_finality(observed_finality: &str, step: &str) -> Result<(), String> {
    if observed_finality.trim() == "FINAL" {
        return Ok(());
    }
    Err(format!(
        "{step} returned non-final finality: {observed_finality}"
    ))
}

fn validate_s14_block_height(observed_block_height: u64, step: &str) -> Result<(), String> {
    if observed_block_height != 0 {
        return Ok(());
    }
    Err(format!("{step} returned block_height=0"))
}
