mod context_build;

use super::super::super::*;
use super::config::{
    DaemonServiceApiRelayP2pConfig, DaemonServiceApiRelayP2pContext,
    DaemonServiceApiRelayP2pTransport, SERVICE_API_RELAY_P2P_CONFIG_ENV,
};
use context_build::{
    advertise_local_peer, build_live_transport, build_swarm_config, non_empty_p2p_config_json,
    parse_daemon_service_api_relay_p2p_config,
};
use kamn_core::PeerGossipFrame;
use std::env;

#[cfg(test)]
pub(crate) use context_build::DaemonServiceApiRelayP2pConfigOverrideGuard;

pub(super) fn resolve_daemon_service_api_relay_p2p_context(
) -> Result<Option<DaemonServiceApiRelayP2pContext>, ConfigError> {
    #[cfg(test)]
    if let Some(override_json) =
        context_build::daemon_service_api_relay_p2p_config_override_json_for_tests()
    {
        return context_build::resolve_override_p2p_context(override_json.as_str());
    }
    resolve_env_p2p_context()
}

pub(super) fn forward_service_api_relay_entry_via_p2p(
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Result<(), String> {
    let recipient_peer_id = recipient_peer_id_for_entry(relay_p2p_context, relay_entry)?;
    let payload = relay_payload_json(relay_entry)?;
    let frame = relay_gossip_frame(
        relay_p2p_context.topic.as_str(),
        relay_p2p_context.local_peer_id.as_str(),
        recipient_peer_id,
        payload.as_str(),
    )?;
    send_p2p_frame(relay_p2p_context, frame)
}

pub(super) fn drain_daemon_service_api_relay_p2p_inbox(
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
    service_api_state_file: Option<&str>,
) -> Result<usize, String> {
    let frames = drained_inbox_frames(relay_p2p_context)?;
    let mut ingested_count = 0_usize;
    for frame in frames {
        if frame_matches_topic(&frame, relay_p2p_context) {
            persist_relay_frame(service_api_state_file, &frame)?;
            ingested_count = ingested_count.saturating_add(1);
        }
    }
    Ok(ingested_count)
}

pub(super) fn resolve_daemon_service_api_relay_p2p_context_from_json(
    raw_json: &str,
) -> Result<DaemonServiceApiRelayP2pContext, ConfigError> {
    let config = parse_daemon_service_api_relay_p2p_config(raw_json)?;
    let swarm_config = build_swarm_config(&config)?;
    let transport = build_live_transport(swarm_config)?;
    build_daemon_service_api_relay_p2p_context(config, transport)
}

pub(super) fn build_daemon_service_api_relay_p2p_context(
    config: DaemonServiceApiRelayP2pConfig,
    transport: DaemonServiceApiRelayP2pTransport,
) -> Result<DaemonServiceApiRelayP2pContext, ConfigError> {
    advertise_local_peer(&config, &transport)?;
    Ok(DaemonServiceApiRelayP2pContext {
        local_peer_id: config.local_peer_id,
        topic: config.topic,
        recipient_peers_by_did: config.recipient_peers_by_did,
        transport,
    })
}

fn resolve_env_p2p_context() -> Result<Option<DaemonServiceApiRelayP2pContext>, ConfigError> {
    let raw = match env::var(SERVICE_API_RELAY_P2P_CONFIG_ENV) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                "{SERVICE_API_RELAY_P2P_CONFIG_ENV} must be valid utf-8 when present"
            )));
        }
    };
    let normalized = non_empty_p2p_config_json(raw.as_str())?;
    resolve_daemon_service_api_relay_p2p_context_from_json(normalized).map(Some)
}

fn recipient_peer_id_for_entry<'a>(
    relay_p2p_context: &'a DaemonServiceApiRelayP2pContext,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Result<&'a str, String> {
    relay_p2p_context
        .recipient_peers_by_did
        .get(relay_entry.recipient_did.as_str())
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "p2p recipient peer mapping missing for recipient_did={}",
                relay_entry.recipient_did
            )
        })
}

fn relay_payload_json(
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Result<String, String> {
    serde_json::to_string(relay_entry)
        .map_err(|error| format!("p2p relay payload serialization failed: {error}"))
}

fn relay_gossip_frame(
    topic: &str,
    local_peer_id: &str,
    recipient_peer_id: &str,
    payload: &str,
) -> Result<PeerGossipFrame, String> {
    PeerGossipFrame::new(topic, local_peer_id, recipient_peer_id, payload)
        .map_err(|error| format!("p2p relay frame build failed: {error}"))
}

fn send_p2p_frame(
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
    frame: PeerGossipFrame,
) -> Result<(), String> {
    relay_p2p_context
        .transport
        .send(frame)
        .map_err(|error| format!("p2p relay send failed: {error}"))
}

fn drained_inbox_frames(
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
) -> Result<Vec<PeerGossipFrame>, String> {
    relay_p2p_context
        .transport
        .drain_inbox(relay_p2p_context.local_peer_id.as_str())
        .map_err(|error| format!("p2p relay inbox drain failed: {error}"))
}

fn frame_matches_topic(
    frame: &PeerGossipFrame,
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
) -> bool {
    frame.topic == relay_p2p_context.topic
}

fn persist_relay_frame(
    service_api_state_file: Option<&str>,
    frame: &PeerGossipFrame,
) -> Result<(), String> {
    let relay_entry = parse_relay_entry(frame.payload.as_str())?;
    crate::service_api_endpoint::upsert_service_api_relayed_message_from_daemon(
        service_api_state_file,
        &relay_entry,
    )
    .map(|_| ())
    .map_err(|error| format!("p2p relay ingress persistence failed: {error}"))
}

fn parse_relay_entry(
    payload: &str,
) -> Result<crate::service_api_endpoint::ServiceApiRelaySpoolEntry, String> {
    serde_json::from_str::<crate::service_api_endpoint::ServiceApiRelaySpoolEntry>(payload)
        .map_err(|error| format!("p2p relay payload parse failed: {error}"))
}

#[cfg(test)]
pub(super) fn set_daemon_service_api_relay_p2p_config_override_for_tests(
    raw_json: Option<&str>,
) -> DaemonServiceApiRelayP2pConfigOverrideGuard {
    context_build::set_daemon_service_api_relay_p2p_config_override_for_tests(raw_json)
}
