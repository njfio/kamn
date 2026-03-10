use super::super::super::super::*;
use super::super::super::service_api_relay_p2p::forward_service_api_relay_entry_via_p2p;
use super::super::{forward_via_http, RelayForwardDecision};

pub(super) fn forward_single_entry(
    relay_p2p_context: Option<
        &super::super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext,
    >,
    relay_route_map: &std::collections::BTreeMap<String, String>,
    relay_signing_private_key_hex: Option<&str>,
    relay_nonce_counter: &mut u64,
    service_api_signature_state_hash: &str,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Result<RelayForwardDecision, ConfigError> {
    let p2p_error = try_p2p_forward(relay_p2p_context, relay_entry);
    if p2p_forward_succeeded(relay_p2p_context, relay_entry, p2p_error.as_ref()) {
        return Ok(RelayForwardDecision::Forwarded);
    }
    if let Some(decision) = retain_pending_or_failed(relay_route_map, p2p_error.clone()) {
        return Ok(decision);
    }
    let signing_key_hex = relay_signing_private_key_hex.ok_or_else(missing_signing_key_error)?;
    forward_via_http(
        relay_route_map,
        relay_entry,
        service_api_signature_state_hash,
        signing_key_hex,
        relay_nonce_counter,
        p2p_error,
    )
}

pub(super) fn retain_pending_or_failed(
    relay_route_map: &std::collections::BTreeMap<String, String>,
    p2p_error: Option<String>,
) -> Option<RelayForwardDecision> {
    if !relay_route_map.is_empty() {
        return None;
    }
    Some(
        p2p_error
            .map(RelayForwardDecision::Failed)
            .unwrap_or(RelayForwardDecision::RetainPending),
    )
}

pub(super) fn p2p_forward_succeeded(
    relay_p2p_context: Option<
        &super::super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext,
    >,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
    p2p_error: Option<&String>,
) -> bool {
    p2p_error.is_none()
        && relay_p2p_context.is_some_and(|context| {
            context
                .recipient_peers_by_did
                .contains_key(relay_entry.recipient_did.as_str())
        })
}

pub(super) fn try_p2p_forward(
    relay_p2p_context: Option<
        &super::super::super::service_api_relay_p2p::DaemonServiceApiRelayP2pContext,
    >,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Option<String> {
    let relay_p2p_context = relay_p2p_context?;
    if !relay_p2p_context
        .recipient_peers_by_did
        .contains_key(relay_entry.recipient_did.as_str())
    {
        return None;
    }
    forward_service_api_relay_entry_via_p2p(relay_p2p_context, relay_entry).err()
}

pub(super) fn missing_signing_key_error() -> ConfigError {
    ConfigError::RuntimeDaemonLifecycle(
        "service api relay forwarding signer key was missing".to_owned(),
    )
}

pub(super) fn combine_forward_errors(p2p_error: Option<String>, http_error: String) -> String {
    match p2p_error {
        Some(existing) => format!("{existing}; http relay forward failed: {http_error}"),
        None => http_error,
    }
}
