use super::{
    base_agent_name, connect_agent, validate_non_empty, DEFAULT_S03_CHANNEL_PAYLOAD,
    DEFAULT_S03_MESSAGE_PAYLOAD, DEFAULT_S04_CREATE_TASK_PAYLOAD, DEFAULT_S04_ESCROW_AMOUNT,
};

pub(super) fn run_live_s03_group_channel_probe() -> Result<(), String> {
    let agent_name = base_agent_name();
    let channel_payload = super::super::env_var_or_default(
        "KAMN_E2E_S03_CHANNEL_PAYLOAD",
        DEFAULT_S03_CHANNEL_PAYLOAD,
    );
    let message_payload = super::super::env_var_or_default(
        "KAMN_E2E_S03_MESSAGE_PAYLOAD",
        DEFAULT_S03_MESSAGE_PAYLOAD,
    );
    let channel = create_channel(
        &format!("{agent_name}-s03-create-channel"),
        &channel_payload,
    )?;
    let message_id = send_message(
        &format!("{agent_name}-s03-send-message"),
        &message_payload,
        "sdk-direct live s03 send-message",
    )?;
    query_channel_message(
        &format!("{agent_name}-s03-query-message"),
        message_id.as_str(),
    )?;
    list_channel_messages(&format!("{agent_name}-s03-list-messages"), channel.as_str())
}

pub(super) fn run_live_s04_task_lifecycle_probe() -> Result<(), String> {
    let agent_name = base_agent_name();
    let create_payload = super::super::env_var_or_default(
        "KAMN_E2E_S04_CREATE_TASK_PAYLOAD",
        DEFAULT_S04_CREATE_TASK_PAYLOAD,
    );
    let task_id = create_task(&format!("{agent_name}-s04-create"), create_payload.as_str())?;
    let escrow_id = fund_task_escrow(&format!("{agent_name}-s04-fund"), task_id.as_str())?;
    accept_task(&format!("{agent_name}-s04-accept"), task_id.as_str())?;
    complete_task(&format!("{agent_name}-s04-complete"), task_id.as_str())?;
    require_release_state(&format!("{agent_name}-s04-release"), escrow_id.as_str())
}

fn create_channel(agent_name: &str, payload: &str) -> Result<String, String> {
    let handle = connect_agent(agent_name, "sdk-direct live s03 connect failed")?;
    let receipt = handle
        .create_channel(payload)
        .map_err(|error| format!("sdk-direct live s03 create-channel failed: {error}"))?;
    validate_non_empty(
        receipt.channel_id.as_str(),
        "sdk-direct live s03 create-channel returned empty channel_id",
    )?;
    validate_non_empty(
        receipt.status.as_str(),
        "sdk-direct live s03 create-channel returned empty status",
    )?;
    Ok(receipt.channel_id)
}

fn send_message(agent_name: &str, payload: &str, context: &str) -> Result<String, String> {
    let handle = connect_agent(agent_name, "sdk-direct live s03 connect failed")?;
    let receipt = handle
        .send_message(payload)
        .map_err(|error| format!("{context} failed: {error}"))?;
    validate_non_empty(
        receipt.message_id.as_str(),
        &format!("{context} returned empty message_id"),
    )?;
    validate_non_empty(
        receipt.status.as_str(),
        &format!("{context} returned empty status"),
    )?;
    Ok(receipt.message_id)
}

fn query_channel_message(agent_name: &str, message_id: &str) -> Result<(), String> {
    let handle = connect_agent(agent_name, "sdk-direct live s03 connect failed")?;
    let queried = handle
        .query_message(message_id)
        .map_err(|error| format!("sdk-direct live s03 query-message failed: {error}"))?;
    validate_live_s03_query_message_response(
        message_id,
        queried.message_id.as_str(),
        queried.status.as_str(),
    )
}

fn list_channel_messages(agent_name: &str, channel_id: &str) -> Result<(), String> {
    let handle = connect_agent(agent_name, "sdk-direct live s03 connect failed")?;
    let listing = handle
        .list_messages(channel_id)
        .map_err(|error| format!("sdk-direct live s03 list-messages failed: {error}"))?;
    validate_live_s03_list_messages_response(channel_id, listing.channel_id.as_str())
}

fn create_task(agent_name: &str, payload: &str) -> Result<String, String> {
    let handle = connect_agent(agent_name, "sdk-direct live s04 connect failed")?;
    let receipt = handle
        .create_task(payload)
        .map_err(|error| format!("sdk-direct live s04 create-task failed: {error}"))?;
    validate_non_empty(
        receipt.task_id.as_str(),
        "sdk-direct live s04 create-task returned empty task_id",
    )?;
    Ok(receipt.task_id)
}

fn fund_task_escrow(agent_name: &str, task_id: &str) -> Result<String, String> {
    let handle = connect_agent(agent_name, "sdk-direct live s04 connect failed")?;
    let payload = format!("{{\"task_id\":\"{task_id}\",\"amount\":{DEFAULT_S04_ESCROW_AMOUNT}}}");
    let receipt = handle
        .fund_escrow(payload.as_str())
        .map_err(|error| format!("sdk-direct live s04 fund-escrow failed: {error}"))?;
    validate_non_empty(
        receipt.escrow_id.as_str(),
        "sdk-direct live s04 fund-escrow returned empty escrow_id",
    )?;
    Ok(receipt.escrow_id)
}

fn accept_task(agent_name: &str, task_id: &str) -> Result<(), String> {
    let handle = connect_agent(agent_name, "sdk-direct live s04 connect failed")?;
    let receipt = handle
        .accept_task(task_id)
        .map_err(|error| format!("sdk-direct live s04 accept-task failed: {error}"))?;
    validate_non_empty(
        receipt.state.as_str(),
        "sdk-direct live s04 accept-task returned empty state",
    )
}

fn complete_task(agent_name: &str, task_id: &str) -> Result<(), String> {
    let handle = connect_agent(agent_name, "sdk-direct live s04 connect failed")?;
    let receipt = handle
        .complete_task(task_id)
        .map_err(|error| format!("sdk-direct live s04 complete-task failed: {error}"))?;
    validate_non_empty(
        receipt.state.as_str(),
        "sdk-direct live s04 complete-task returned empty state",
    )
}

fn require_release_state(agent_name: &str, escrow_id: &str) -> Result<(), String> {
    let handle = connect_agent(agent_name, "sdk-direct live s04 connect failed")?;
    let receipt = handle
        .release_escrow(escrow_id)
        .map_err(|error| format!("sdk-direct live s04 release-escrow failed: {error}"))?;
    validate_non_empty(
        receipt.state.as_str(),
        "sdk-direct live s04 release-escrow returned empty state",
    )
}

pub(super) fn validate_live_s03_query_message_response(
    expected_message_id: &str,
    queried_message_id: &str,
    queried_status: &str,
) -> Result<(), String> {
    if queried_message_id != expected_message_id {
        return Err(format!(
            "sdk-direct live s03 query-message returned mismatched message_id: expected={expected_message_id}, got={queried_message_id}"
        ));
    }
    validate_non_empty(
        queried_status,
        "sdk-direct live s03 query-message returned empty status",
    )
}

pub(super) fn validate_live_s03_list_messages_response(
    expected_channel_id: &str,
    listed_channel_id: &str,
) -> Result<(), String> {
    if listed_channel_id != expected_channel_id {
        return Err(format!(
            "sdk-direct live s03 list-messages returned mismatched channel_id: expected={expected_channel_id}, got={listed_channel_id}"
        ));
    }
    Ok(())
}
