use super::{
    base_agent_name, connect_agent, validate_non_empty, DEFAULT_S02_MESSAGE_PAYLOAD,
    DEFAULT_S02_REPLY_PAYLOAD,
};
use kamn_agent_lib::KamnAgentHandle;

pub(super) fn run_live_s01_discovery_probe() -> Result<(), String> {
    let handle = connect_agent(
        base_agent_name().as_str(),
        "sdk-direct live discovery connect failed",
    )?;
    validate_non_empty(
        handle.identity().did().as_str(),
        "sdk-direct live discovery failed: empty DID",
    )?;
    let health = handle
        .health()
        .map_err(|error| format!("sdk-direct live discovery health check failed: {error}"))?;
    validate_non_empty(
        health.status.as_str(),
        "sdk-direct live discovery failed: empty health status",
    )
}

pub(super) fn run_live_s02_direct_message_probe() -> Result<(), String> {
    let agent_name = base_agent_name();
    let (message_payload, reply_payload) = s02_payloads();
    run_message_roundtrip(
        agent_name.as_str(),
        "send",
        message_payload.as_str(),
        "sdk-direct live s02 send-message",
    )?;
    run_message_roundtrip(
        agent_name.as_str(),
        "reply",
        reply_payload.as_str(),
        "sdk-direct live s02 reply send-message",
    )?;
    Ok(())
}

fn s02_payloads() -> (String, String) {
    let message_payload = super::super::env_var_or_default(
        "KAMN_E2E_S02_MESSAGE_PAYLOAD",
        DEFAULT_S02_MESSAGE_PAYLOAD,
    );
    let reply_payload =
        super::super::env_var_or_default("KAMN_E2E_S02_REPLY_PAYLOAD", DEFAULT_S02_REPLY_PAYLOAD);
    (message_payload, reply_payload)
}

fn run_message_roundtrip(
    agent_name: &str,
    suffix: &str,
    payload: &str,
    context: &str,
) -> Result<(), String> {
    let sent = send_message_receipt(&format!("{agent_name}-s02-{suffix}"), payload, context)?;
    let query_suffix = if suffix == "send" {
        "query"
    } else {
        "query-reply"
    };
    let query_context = context.replace(" send-message", " query-message");
    query_message_receipt(
        &format!("{agent_name}-s02-{query_suffix}"),
        sent.0.as_str(),
        query_context.as_str(),
    )
}

fn send_message_receipt(
    agent_name: &str,
    payload: &str,
    context: &str,
) -> Result<(String, String), String> {
    let handle = connect_agent(
        agent_name,
        &format!("{} connect failed", context.replace(" send-message", "")),
    )?;
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
    Ok((receipt.message_id, receipt.status))
}

fn query_message_receipt(agent_name: &str, message_id: &str, context: &str) -> Result<(), String> {
    let handle = connect_agent(
        agent_name,
        &format!("{} connect failed", context.replace(" query-message", "")),
    )?;
    let queried = handle
        .query_message(message_id)
        .map_err(|error| format!("{context} failed: {error}"))?;
    validate_matching_message_id(context, message_id, queried.message_id.as_str())?;
    validate_non_empty(
        queried.status.as_str(),
        &format!("{context} returned empty status"),
    )
}

fn validate_matching_message_id(context: &str, expected: &str, actual: &str) -> Result<(), String> {
    if actual != expected {
        return Err(format!(
            "{context} returned mismatched message_id: expected={expected}, got={actual}"
        ));
    }
    Ok(())
}

#[allow(dead_code)]
fn _type_anchor(_: KamnAgentHandle) {}
