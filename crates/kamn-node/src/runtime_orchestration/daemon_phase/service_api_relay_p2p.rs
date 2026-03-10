mod config;
#[cfg(test)]
mod test_support;
mod transport;

use super::super::*;

pub(super) type DaemonServiceApiRelayP2pContext = config::DaemonServiceApiRelayP2pContext;

pub(super) fn resolve_daemon_service_api_relay_p2p_context(
) -> Result<Option<DaemonServiceApiRelayP2pContext>, ConfigError> {
    transport::resolve_daemon_service_api_relay_p2p_context()
}

pub(super) fn forward_service_api_relay_entry_via_p2p(
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Result<(), String> {
    transport::forward_service_api_relay_entry_via_p2p(relay_p2p_context, relay_entry)
}

pub(super) fn drain_daemon_service_api_relay_p2p_inbox(
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
    service_api_state_file: Option<&str>,
) -> Result<usize, String> {
    transport::drain_daemon_service_api_relay_p2p_inbox(relay_p2p_context, service_api_state_file)
}

#[cfg(test)]
pub(super) const SERVICE_API_RELAY_P2P_DEFAULT_TOPIC_FOR_TEST: &str =
    test_support::SERVICE_API_RELAY_P2P_DEFAULT_TOPIC_FOR_TEST;

#[cfg(test)]
pub(super) fn resolve_daemon_service_api_relay_p2p_in_memory_context_from_json_for_test(
    raw_json: &str,
    shared_transport: std::sync::Arc<kamn_core::InMemoryPeerLifecycleTransport>,
) -> Result<DaemonServiceApiRelayP2pContext, ConfigError> {
    test_support::resolve_daemon_service_api_relay_p2p_in_memory_context_from_json_for_test(
        raw_json,
        shared_transport,
    )
}

#[cfg(test)]
pub(super) fn set_daemon_service_api_relay_p2p_config_override_for_test(
    raw_json: Option<&str>,
) -> transport::DaemonServiceApiRelayP2pConfigOverrideGuard {
    test_support::set_daemon_service_api_relay_p2p_config_override_for_test(raw_json)
}

#[cfg(test)]
pub(super) fn forward_service_api_relay_entry_via_p2p_for_test(
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
    relay_entry: &crate::service_api_endpoint::ServiceApiRelaySpoolEntry,
) -> Result<(), String> {
    test_support::forward_service_api_relay_entry_via_p2p_for_test(relay_p2p_context, relay_entry)
}

#[cfg(test)]
pub(super) fn drain_daemon_service_api_relay_p2p_inbox_for_test(
    relay_p2p_context: &DaemonServiceApiRelayP2pContext,
    service_api_state_file: Option<&str>,
) -> Result<usize, String> {
    test_support::drain_daemon_service_api_relay_p2p_inbox_for_test(
        relay_p2p_context,
        service_api_state_file,
    )
}
