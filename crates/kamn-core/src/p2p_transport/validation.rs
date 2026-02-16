use super::P2pTransportError;

pub(super) fn validate_peer_id(peer_id: &str) -> Result<(), P2pTransportError> {
    if peer_id.trim().is_empty() {
        return Err(P2pTransportError::InvalidPeerId);
    }
    Ok(())
}

pub(super) fn validate_topic(topic: &str) -> Result<(), P2pTransportError> {
    let trimmed = topic.trim();
    if trimmed.is_empty()
        || trimmed.contains('|')
        || trimmed.contains('\n')
        || trimmed.contains('\r')
    {
        return Err(P2pTransportError::InvalidTopic);
    }
    Ok(())
}

pub(super) fn validate_swarm_listen_address(listen_address: &str) -> Result<(), P2pTransportError> {
    if is_supported_swarm_multiaddr(listen_address) {
        return Ok(());
    }
    Err(P2pTransportError::InvalidSwarmListenAddress)
}

pub(super) fn validate_swarm_bootstrap_peer_address(
    address: &str,
) -> Result<(), P2pTransportError> {
    if is_supported_swarm_multiaddr(address) {
        return Ok(());
    }
    Err(P2pTransportError::InvalidSwarmBootstrapPeerAddress(
        address.to_owned(),
    ))
}

pub(super) fn is_supported_swarm_multiaddr(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return false;
    }
    let has_address_prefix = trimmed.starts_with("/ip4/")
        || trimmed.starts_with("/ip6/")
        || trimmed.starts_with("/dns/")
        || trimmed.starts_with("/dns4/")
        || trimmed.starts_with("/dns6/");
    has_address_prefix
        && trimmed.contains("/tcp/")
        && !trimmed.contains('\n')
        && !trimmed.contains('\r')
}
