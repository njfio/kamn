use super::super::super::*;
use super::config::{
    normalize_daemon_service_api_relay_p2p_config, DaemonServiceApiRelayP2pConfigInput,
    DaemonServiceApiRelayP2pContext, DaemonServiceApiRelayP2pTransport,
    SERVICE_API_RELAY_P2P_CONFIG_ENV, SERVICE_API_RELAY_P2P_DEFAULT_TOPIC,
};
use super::transport::{
    build_daemon_service_api_relay_p2p_context, drain_daemon_service_api_relay_p2p_inbox,
    forward_service_api_relay_entry_via_p2p,
    set_daemon_service_api_relay_p2p_config_override_for_tests,
    DaemonServiceApiRelayP2pConfigOverrideGuard,
};
use std::sync::Arc;

pub(crate) const SERVICE_API_RELAY_P2P_DEFAULT_TOPIC_FOR_TEST: &str =
    SERVICE_API_RELAY_P2P_DEFAULT_TOPIC;

pub(crate) fn resolve_daemon_service_api_relay_p2p_in_memory_context_from_json_for_test(
    raw_json: &str,
    shared_transport: Arc<kamn_core::InMemoryPeerLifecycleTransport>,
) -> Result<DaemonServiceApiRelayP2pContext, ConfigError> {
    let parsed = serde_json::from_str::<DaemonServiceApiRelayP2pConfigInput>(raw_json).map_err(
        |error| {
            ConfigError::RuntimeDaemonLifecycle(format!(
                "{SERVICE_API_RELAY_P2P_CONFIG_ENV} must be a JSON object with local_peer_id/listen_address/bootstrap_peers/topic/recipient_peers_by_did: {error}"
            ))
        },
    )?;
    let config = normalize_daemon_service_api_relay_p2p_config(parsed)?;
    let transport: DaemonServiceApiRelayP2pTransport = shared_transport;
    build_daemon_service_api_relay_p2p_context(config, transport)
}

pub(crate) fn set_daemon_service_api_relay_p2p_config_override_for_test(
    raw_json: Option<&str>,
) -> DaemonServiceApiRelayP2pConfigOverrideGuard {
    set_daemon_service_api_relay_p2p_config_override_for_tests(raw_json)
}

pub(crate) fn forward_service_api_relay_entry_via_p2p_for_test(
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Result<(), String> {
    forward_service_api_relay_entry_via_p2p(relay_p2p_context, relay_entry)
}

pub(crate) fn drain_daemon_service_api_relay_p2p_inbox_for_test(
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
    service_api_state_file: Option<&str>,
) -> Result<usize, String> {
    drain_daemon_service_api_relay_p2p_inbox(relay_p2p_context, service_api_state_file)
}
