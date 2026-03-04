use super::{ConfigError, PeerLifecycleEvent, ProposalCandidate, RejoinAttempt};

fn split_four_segments(value: &str) -> Option<[&str; 4]> {
    let segments = value.split('|').collect::<Vec<&str>>();
    if segments.len() != 4 {
        return None;
    }
    Some([segments[0], segments[1], segments[2], segments[3]])
}

pub(super) fn parse_state_version_arg(value: &str) -> Result<u64, ConfigError> {
    let state_version = value
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidExpectedStateVersion(value.to_owned()))?;
    if state_version == 0 {
        return Err(ConfigError::InvalidExpectedStateVersion(value.to_owned()));
    }
    Ok(state_version)
}

pub(super) fn parse_proposal_candidate(value: &str) -> Result<ProposalCandidate, ConfigError> {
    let parts = split_four_segments(value)
        .ok_or_else(|| ConfigError::InvalidProposalArgument(value.to_owned()))?;
    let nonce = parts[2]
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidProposalArgument(value.to_owned()))?;
    ProposalCandidate::new(parts[0], parts[1], nonce, parts[3])
        .map_err(|error| ConfigError::RuntimePlanner(error.to_string()))
}

pub(super) fn parse_rejoin_attempt(value: &str) -> Result<RejoinAttempt, ConfigError> {
    let parts = split_four_segments(value)
        .ok_or_else(|| ConfigError::InvalidRejoinAttemptArgument(value.to_owned()))?;
    let state_version = parts[1]
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidRejoinAttemptArgument(value.to_owned()))?;
    RejoinAttempt::new(parts[0], state_version, parts[2], parts[3])
        .map_err(|_| ConfigError::InvalidRejoinAttemptArgument(value.to_owned()))
}

pub(super) fn parse_daemon_control_arg(value: &str) -> Result<u64, ConfigError> {
    let parsed = value
        .parse::<u64>()
        .map_err(|_| ConfigError::InvalidDaemonControlArgument(value.to_owned()))?;
    if parsed == 0 {
        return Err(ConfigError::InvalidDaemonControlArgument(value.to_owned()));
    }
    Ok(parsed)
}

pub(super) fn parse_daemon_lifecycle_event(value: &str) -> Result<PeerLifecycleEvent, ConfigError> {
    match value {
        "start-connect" => Ok(PeerLifecycleEvent::StartConnect),
        "handshake-succeeded" => Ok(PeerLifecycleEvent::HandshakeSucceeded),
        "heartbeat-missed" => Ok(PeerLifecycleEvent::HeartbeatMissed),
        "heartbeat-restored" => Ok(PeerLifecycleEvent::HeartbeatRestored),
        "disconnect" => Ok(PeerLifecycleEvent::Disconnect),
        "rejoin" => Ok(PeerLifecycleEvent::Rejoin),
        _ => Err(ConfigError::InvalidDaemonLifecycleEvent(value.to_owned())),
    }
}
