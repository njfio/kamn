use super::super::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Peer lifecycle state.
pub enum PeerLifecycleState {
    /// Disconnected.
    Disconnected,
    /// Connecting.
    Connecting,
    /// Active.
    Active,
    /// Degraded.
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Peer lifecycle event.
pub enum PeerLifecycleEvent {
    /// Start connect.
    StartConnect,
    /// Handshake succeeded.
    HandshakeSucceeded,
    /// Heartbeat missed.
    HeartbeatMissed,
    /// Heartbeat restored.
    HeartbeatRestored,
    /// Disconnect.
    Disconnect,
    /// Rejoin.
    Rejoin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime lifecycle error.
pub enum RuntimeLifecycleError {
    /// Invalid peer id.
    InvalidPeerId,
    /// Invalid transition.
    InvalidTransition {
        /// From.
        from: PeerLifecycleState,
        /// Event.
        event: PeerLifecycleEvent,
    },
}

impl RuntimeLifecycleError {
    /// Handles reason code.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidPeerId => "runtime_peer_id_invalid",
            Self::InvalidTransition { .. } => "runtime_peer_transition_invalid",
        }
    }
}

impl Display for RuntimeLifecycleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPeerId => write!(f, "runtime peer id cannot be empty"),
            Self::InvalidTransition { from, event } => {
                write!(
                    f,
                    "invalid peer lifecycle transition from {from:?} via {event:?}"
                )
            }
        }
    }
}

impl Error for RuntimeLifecycleError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Peer lifecycle.
pub struct PeerLifecycle {
    peer_id: String,
    state: PeerLifecycleState,
}

impl PeerLifecycle {
    /// Handles new.
    pub fn new(peer_id: &str) -> Result<Self, RuntimeLifecycleError> {
        if peer_id.trim().is_empty() {
            return Err(RuntimeLifecycleError::InvalidPeerId);
        }
        Ok(Self {
            peer_id: peer_id.to_owned(),
            state: PeerLifecycleState::Disconnected,
        })
    }

    /// Handles peer id.
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    /// Handles state.
    pub fn state(&self) -> PeerLifecycleState {
        self.state
    }

    /// Handles transition.
    pub fn transition(
        &mut self,
        event: PeerLifecycleEvent,
    ) -> Result<PeerLifecycleState, RuntimeLifecycleError> {
        let Some(next_state) = next_peer_state(self.state, event) else {
            return Err(RuntimeLifecycleError::InvalidTransition {
                from: self.state,
                event,
            });
        };
        self.state = next_state;
        Ok(next_state)
    }
}

fn next_peer_state(
    from: PeerLifecycleState,
    event: PeerLifecycleEvent,
) -> Option<PeerLifecycleState> {
    match (from, event) {
        (PeerLifecycleState::Disconnected, event) => reconnect_state(event),
        (PeerLifecycleState::Connecting, event) => connecting_state(event),
        (PeerLifecycleState::Active, event) => active_state(event),
        (PeerLifecycleState::Degraded, event) => degraded_state(event),
    }
}

fn reconnect_state(event: PeerLifecycleEvent) -> Option<PeerLifecycleState> {
    match event {
        PeerLifecycleEvent::StartConnect | PeerLifecycleEvent::Rejoin => {
            Some(PeerLifecycleState::Connecting)
        }
        _ => None,
    }
}

fn connecting_state(event: PeerLifecycleEvent) -> Option<PeerLifecycleState> {
    match event {
        PeerLifecycleEvent::HandshakeSucceeded => Some(PeerLifecycleState::Active),
        PeerLifecycleEvent::Disconnect => Some(PeerLifecycleState::Disconnected),
        _ => None,
    }
}

fn active_state(event: PeerLifecycleEvent) -> Option<PeerLifecycleState> {
    match event {
        PeerLifecycleEvent::HeartbeatMissed => Some(PeerLifecycleState::Degraded),
        PeerLifecycleEvent::Disconnect => Some(PeerLifecycleState::Disconnected),
        _ => None,
    }
}

fn degraded_state(event: PeerLifecycleEvent) -> Option<PeerLifecycleState> {
    match event {
        PeerLifecycleEvent::HeartbeatRestored => Some(PeerLifecycleState::Active),
        PeerLifecycleEvent::Disconnect => Some(PeerLifecycleState::Disconnected),
        _ => None,
    }
}
