use super::super::super::super::*;
use super::super::config::{
    normalize_daemon_service_api_relay_p2p_config, DaemonServiceApiRelayP2pConfig,
    DaemonServiceApiRelayP2pConfigInput, DaemonServiceApiRelayP2pContext,
    DaemonServiceApiRelayP2pTransport, SERVICE_API_RELAY_P2P_CONFIG_ENV,
    SERVICE_API_RELAY_P2P_HARNESS_TICK_BUDGET,
};
use kamn_core::{
    Libp2pLivePeerLifecycleTransport, NodeRole, P2pSwarmDeterministicConfig, P2pSwarmHarnessMode,
    PeerDiscoveryRecord,
};
use std::sync::Arc;

pub(super) fn resolve_override_p2p_context(
    override_json: &str,
) -> Result<Option<DaemonServiceApiRelayP2pContext>, ConfigError> {
    super::resolve_daemon_service_api_relay_p2p_context_from_json(override_json).map(Some)
}

pub(super) fn non_empty_p2p_config_json(raw: &str) -> Result<&str, ConfigError> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_P2P_CONFIG_ENV} must not be empty when present"
        )));
    }
    Ok(normalized)
}

pub(super) fn parse_daemon_service_api_relay_p2p_config(
    raw_json: &str,
) -> Result<DaemonServiceApiRelayP2pConfig, ConfigError> {
    let parsed = serde_json::from_str::<DaemonServiceApiRelayP2pConfigInput>(raw_json)
        .map_err(p2p_config_json_error)?;
    normalize_daemon_service_api_relay_p2p_config(parsed)
}

fn p2p_config_json_error(error: serde_json::Error) -> ConfigError {
    ConfigError::RuntimeDaemonLifecycle(format!(
        "{SERVICE_API_RELAY_P2P_CONFIG_ENV} must be a JSON object with local_peer_id/listen_address/bootstrap_peers/topic/recipient_peers_by_did: {error}"
    ))
}

pub(super) fn build_swarm_config(
    config: &DaemonServiceApiRelayP2pConfig,
) -> Result<P2pSwarmDeterministicConfig, ConfigError> {
    P2pSwarmDeterministicConfig::new(
        config.local_peer_id.as_str(),
        config.listen_address.as_str(),
        config.bootstrap_peers.clone(),
        vec![config.topic.clone()],
        SERVICE_API_RELAY_P2P_HARNESS_TICK_BUDGET,
    )
    .map_err(|error| {
        ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_P2P_CONFIG_ENV} swarm config validation failed: {error}"
        ))
    })
}

pub(super) fn build_live_transport(
    swarm_config: P2pSwarmDeterministicConfig,
) -> Result<DaemonServiceApiRelayP2pTransport, ConfigError> {
    Libp2pLivePeerLifecycleTransport::new(swarm_config, P2pSwarmHarnessMode::DryRun)
        .map(|transport| Arc::new(transport) as DaemonServiceApiRelayP2pTransport)
        .map_err(|error| {
            ConfigError::RuntimeDaemonLifecycle(format!(
                "{SERVICE_API_RELAY_P2P_CONFIG_ENV} transport initialization failed: {error}"
            ))
        })
}

pub(super) fn advertise_local_peer(
    config: &DaemonServiceApiRelayP2pConfig,
    transport: &DaemonServiceApiRelayP2pTransport,
) -> Result<(), ConfigError> {
    let local_record = PeerDiscoveryRecord::new(
        config.local_peer_id.as_str(),
        NodeRole::Processor,
        vec![config.topic.clone()],
    )
    .map_err(|error| {
        ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_P2P_CONFIG_ENV} local peer discovery record invalid: {error}"
        ))
    })?;
    transport.advertise(local_record).map_err(|error| {
        ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_P2P_CONFIG_ENV} local peer advertise failed: {error}"
        ))
    })
}
