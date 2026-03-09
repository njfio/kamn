use super::{
    config::{BRIDGE_READ_SCOPE, BRIDGE_WRITE_SCOPE},
    state::{build_auth, remember_message_id},
    task_escrow::deterministic_u64_tag,
    LiveTransportKamnClient,
};
use crate::{
    bridge::target_network, BridgeId, BridgeStatus, MessageId, SdkError, ServiceBridgeStatus,
    ServiceBridgeSubmission, ServiceRequestAuth,
};

pub(crate) fn bridge_read_auth(
    client: &LiveTransportKamnClient,
) -> Result<ServiceRequestAuth, SdkError> {
    build_auth(
        &client.state,
        &client.config,
        &client.config.requester_did,
        "",
        Some(BRIDGE_READ_SCOPE),
    )
}

pub(crate) fn bridge_write_auth(
    client: &LiveTransportKamnClient,
    body: &str,
) -> Result<ServiceRequestAuth, SdkError> {
    build_auth(
        &client.state,
        &client.config,
        &client.config.requester_did,
        body,
        Some(BRIDGE_WRITE_SCOPE),
    )
}

pub(crate) fn bridge_submit_payload(
    service_message_id: &str,
    target: &str,
) -> Result<String, SdkError> {
    let target = target_network(target)?;
    Ok(serde_json::json!({
        "source_message_id": service_message_id,
        "target_network": target,
    })
    .to_string())
}

pub(crate) fn remember_bridge_id(
    client: &LiveTransportKamnClient,
    service_bridge_id: &str,
) -> Result<BridgeId, SdkError> {
    validate_bridge_id(service_bridge_id)?;
    let alias = deterministic_u64_tag(service_bridge_id);
    let mut guard = client
        .state
        .lock()
        .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
    if let Some(existing) = guard.bridge_ids.get(&alias) {
        if existing != service_bridge_id {
            return Err(SdkError::Conflict(
                "service bridge id collision detected in sdk bridge alias map",
            ));
        }
    } else {
        guard.bridge_ids.insert(alias, service_bridge_id.to_owned());
    }
    Ok(BridgeId(alias))
}

pub(crate) fn resolve_service_bridge_id(
    client: &LiveTransportKamnClient,
    bridge_id: &BridgeId,
) -> Result<String, SdkError> {
    let guard = client
        .state
        .lock()
        .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
    guard
        .bridge_ids
        .get(&bridge_id.0)
        .cloned()
        .ok_or_else(|| bridge_not_found(bridge_id))
}

pub(crate) fn resolve_service_message_id(
    client: &LiveTransportKamnClient,
    message_id: &MessageId,
) -> Result<String, SdkError> {
    let guard = client
        .state
        .lock()
        .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
    guard
        .message_ids
        .get(&message_id.0)
        .cloned()
        .ok_or_else(|| message_not_found(message_id))
}

pub(crate) fn bridge_status_from_submission(
    client: &LiveTransportKamnClient,
    submission: ServiceBridgeSubmission,
) -> Result<BridgeStatus, SdkError> {
    let bridge_id = remember_bridge_id(client, submission.bridge_id.as_str())?;
    Ok(BridgeStatus::submitted(&bridge_id))
}

pub(crate) fn bridge_status_from_service(
    client: &LiveTransportKamnClient,
    bridge_id: &BridgeId,
    status: ServiceBridgeStatus,
) -> Result<BridgeStatus, SdkError> {
    let target_message_id = if status.target_message_id.trim().is_empty() {
        None
    } else {
        Some(remember_message_id(
            &client.state,
            status.target_message_id.as_str(),
        )?)
    };
    let forward_tx_hash = if status.forward_tx_hash.trim().is_empty() {
        None
    } else {
        Some(status.forward_tx_hash)
    };
    Ok(BridgeStatus {
        bridge_id: bridge_id.clone(),
        bridge_status: status.bridge_status,
        target_message_id,
        forward_tx_hash,
    })
}

pub(crate) fn bridge_not_found(bridge_id: &BridgeId) -> SdkError {
    SdkError::NotFound {
        entity: "bridge",
        id: bridge_id.0.to_string(),
    }
}

pub(crate) fn message_not_found(message_id: &MessageId) -> SdkError {
    SdkError::NotFound {
        entity: "message",
        id: message_id.0.to_string(),
    }
}

fn validate_bridge_id(service_bridge_id: &str) -> Result<(), SdkError> {
    if service_bridge_id.trim().is_empty() {
        return Err(SdkError::TransportFailure(
            "service returned empty bridge_id in bridge response",
        ));
    }
    Ok(())
}
