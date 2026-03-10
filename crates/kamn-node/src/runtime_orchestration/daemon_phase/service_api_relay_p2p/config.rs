use super::super::super::*;
use kamn_core::PeerLifecycleTransport;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub(super) const SERVICE_API_RELAY_P2P_CONFIG_ENV: &str = "KAMN_SERVICE_API_RELAY_P2P_CONFIG_JSON";
pub(super) const SERVICE_API_RELAY_P2P_DEFAULT_TOPIC: &str = "messages";
pub(super) const SERVICE_API_RELAY_P2P_HARNESS_TICK_BUDGET: u16 = 3;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(super) struct DaemonServiceApiRelayP2pConfigInput {
    pub(crate) local_peer_id: String,
    pub(super) listen_address: String,
    #[serde(default)]
    pub(super) bootstrap_peers: Vec<String>,
    #[serde(default = "default_service_api_relay_p2p_topic")]
    pub(crate) topic: String,
    #[serde(default)]
    pub(crate) recipient_peers_by_did: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DaemonServiceApiRelayP2pConfig {
    pub(crate) local_peer_id: String,
    pub(super) listen_address: String,
    pub(super) bootstrap_peers: Vec<String>,
    pub(crate) topic: String,
    pub(crate) recipient_peers_by_did: BTreeMap<String, String>,
}

pub(super) type DaemonServiceApiRelayP2pTransport = Arc<dyn PeerLifecycleTransport + Send + Sync>;

#[derive(Clone)]
pub(crate) struct DaemonServiceApiRelayP2pContext {
    pub(crate) local_peer_id: String,
    pub(crate) topic: String,
    pub(crate) recipient_peers_by_did: BTreeMap<String, String>,
    pub(crate) transport: DaemonServiceApiRelayP2pTransport,
}

pub(super) fn normalize_daemon_service_api_relay_p2p_config(
    config: DaemonServiceApiRelayP2pConfigInput,
) -> Result<DaemonServiceApiRelayP2pConfig, ConfigError> {
    let local_peer_id = non_empty_config_value(config.local_peer_id.as_str(), "local_peer_id")?;
    let listen_address = non_empty_config_value(config.listen_address.as_str(), "listen_address")?;
    let topic = non_empty_config_value(config.topic.as_str(), "topic")?;
    let bootstrap_peers = normalize_bootstrap_peers(config.bootstrap_peers, listen_address)?;
    let recipient_peers_by_did = normalize_recipient_map(config.recipient_peers_by_did)?;
    Ok(DaemonServiceApiRelayP2pConfig {
        local_peer_id: local_peer_id.to_owned(),
        listen_address: listen_address.to_owned(),
        bootstrap_peers,
        topic: topic.to_owned(),
        recipient_peers_by_did,
    })
}

fn non_empty_config_value<'a>(value: &'a str, field: &str) -> Result<&'a str, ConfigError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_P2P_CONFIG_ENV} {field} must not be empty"
        )));
    }
    Ok(normalized)
}

fn normalize_bootstrap_peers(
    bootstrap_peers: Vec<String>,
    listen_address: &str,
) -> Result<Vec<String>, ConfigError> {
    let mut normalized = BTreeSet::new();
    for peer in bootstrap_peers {
        let normalized_peer = peer.trim();
        if normalized_peer.is_empty() {
            return Err(ConfigError::RuntimeDaemonLifecycle(format!(
                "{SERVICE_API_RELAY_P2P_CONFIG_ENV} bootstrap_peers must not include empty entries"
            )));
        }
        normalized.insert(normalized_peer.to_owned());
    }
    if normalized.is_empty() {
        normalized.insert(listen_address.to_owned());
    }
    Ok(normalized.into_iter().collect())
}

fn normalize_recipient_map(
    recipient_peers_by_did: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, ConfigError> {
    let mut normalized = BTreeMap::new();
    for (recipient_did, peer_id) in recipient_peers_by_did {
        let (normalized_recipient_did, normalized_peer_id) =
            normalize_recipient_peer_entry(recipient_did.as_str(), peer_id.as_str())?;
        normalized.insert(normalized_recipient_did, normalized_peer_id);
    }
    Ok(normalized)
}

fn normalize_recipient_peer_entry(
    recipient_did: &str,
    peer_id: &str,
) -> Result<(String, String), ConfigError> {
    let normalized_recipient_did = recipient_did.trim();
    if normalized_recipient_did.is_empty() {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_P2P_CONFIG_ENV} recipient_peers_by_did contains an empty recipient DID key"
        )));
    }
    let normalized_peer_id = peer_id.trim();
    if normalized_peer_id.is_empty() {
        return Err(ConfigError::RuntimeDaemonLifecycle(format!(
            "{SERVICE_API_RELAY_P2P_CONFIG_ENV} recipient_peers_by_did contains an empty peer id for recipient={normalized_recipient_did}"
        )));
    }
    Ok((
        normalized_recipient_did.to_owned(),
        normalized_peer_id.to_owned(),
    ))
}

fn default_service_api_relay_p2p_topic() -> String {
    SERVICE_API_RELAY_P2P_DEFAULT_TOPIC.to_owned()
}
