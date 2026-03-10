use super::super::super::*;

pub(super) fn build_peer_lifecycle_summary(
    daemon_peer_id: Option<String>,
    daemon_lifecycle_events: Vec<PeerLifecycleEvent>,
) -> Result<(Option<String>, Option<String>, Option<Vec<String>>), ConfigError> {
    match daemon_peer_id {
        Some(peer_id) => build_present_peer_summary(peer_id, daemon_lifecycle_events),
        None => Ok((None, None, None)),
    }
}

fn build_present_peer_summary(
    peer_id: String,
    daemon_lifecycle_events: Vec<PeerLifecycleEvent>,
) -> Result<(Option<String>, Option<String>, Option<Vec<String>>), ConfigError> {
    let mut lifecycle = PeerLifecycle::new(&peer_id)
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
    let mut applied_events = Vec::with_capacity(daemon_lifecycle_events.len());
    for event in daemon_lifecycle_events {
        lifecycle
            .transition(event)
            .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
        applied_events.push(daemon_lifecycle_event_as_str(event).to_owned());
    }
    Ok((
        Some(peer_id),
        Some(peer_lifecycle_state_as_str(lifecycle.state()).to_owned()),
        Some(applied_events),
    ))
}

fn daemon_lifecycle_event_as_str(event: PeerLifecycleEvent) -> &'static str {
    match event {
        PeerLifecycleEvent::StartConnect => "start-connect",
        PeerLifecycleEvent::HandshakeSucceeded => "handshake-succeeded",
        PeerLifecycleEvent::HeartbeatMissed => "heartbeat-missed",
        PeerLifecycleEvent::HeartbeatRestored => "heartbeat-restored",
        PeerLifecycleEvent::Disconnect => "disconnect",
        PeerLifecycleEvent::Rejoin => "rejoin",
    }
}

fn peer_lifecycle_state_as_str(state: PeerLifecycleState) -> &'static str {
    match state {
        PeerLifecycleState::Disconnected => "disconnected",
        PeerLifecycleState::Connecting => "connecting",
        PeerLifecycleState::Active => "active",
        PeerLifecycleState::Degraded => "degraded",
    }
}
