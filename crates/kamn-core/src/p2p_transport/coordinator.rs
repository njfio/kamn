use crate::config::NodeRole;
use crate::runtime::{PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState};

use super::validation::validate_topic;
use super::{P2pTransportError, PeerDiscoveryRecord, PeerGossipFrame, PeerLifecycleTransport};

/// Coordinator that wires lifecycle transitions to transport discovery and gossip operations.
#[derive(Debug, Clone)]
pub struct PeerLifecycleTransportCoordinator<T: PeerLifecycleTransport> {
    local_peer_id: String,
    local_role: NodeRole,
    lifecycle: PeerLifecycle,
    transport: T,
}

impl<T: PeerLifecycleTransport> PeerLifecycleTransportCoordinator<T> {
    /// Builds a coordinator for the local peer identity.
    pub fn new(
        local_peer_id: &str,
        local_role: NodeRole,
        transport: T,
    ) -> Result<Self, P2pTransportError> {
        let lifecycle = PeerLifecycle::new(local_peer_id)?;
        Ok(Self {
            local_peer_id: local_peer_id.to_owned(),
            local_role,
            lifecycle,
            transport,
        })
    }

    /// Returns the current lifecycle state.
    pub fn lifecycle_state(&self) -> PeerLifecycleState {
        self.lifecycle.state()
    }

    /// Executes connect + handshake transitions and advertises the peer for gossip discovery.
    pub fn connect_and_advertise(
        &mut self,
        gossip_topics: Vec<String>,
    ) -> Result<PeerLifecycleState, P2pTransportError> {
        self.lifecycle
            .transition(PeerLifecycleEvent::StartConnect)?;
        let next = self
            .lifecycle
            .transition(PeerLifecycleEvent::HandshakeSucceeded)?;
        let record =
            PeerDiscoveryRecord::new(&self.local_peer_id, self.local_role.clone(), gossip_topics)?;
        self.transport.advertise(record)?;
        Ok(next)
    }

    /// Transitions the peer to disconnected lifecycle state.
    pub fn disconnect(&mut self) -> Result<PeerLifecycleState, P2pTransportError> {
        Ok(self.lifecycle.transition(PeerLifecycleEvent::Disconnect)?)
    }

    /// Applies a deterministic lifecycle transition from a live transport event signal.
    pub fn apply_live_transport_signal(
        &mut self,
        event: PeerLifecycleEvent,
    ) -> Result<PeerLifecycleState, P2pTransportError> {
        match event {
            PeerLifecycleEvent::HandshakeSucceeded => {
                if self.lifecycle.state() == PeerLifecycleState::Disconnected {
                    self.lifecycle
                        .transition(PeerLifecycleEvent::StartConnect)?;
                }
                Ok(self
                    .lifecycle
                    .transition(PeerLifecycleEvent::HandshakeSucceeded)?)
            }
            PeerLifecycleEvent::HeartbeatMissed => Ok(self
                .lifecycle
                .transition(PeerLifecycleEvent::HeartbeatMissed)?),
            PeerLifecycleEvent::HeartbeatRestored => Ok(self
                .lifecycle
                .transition(PeerLifecycleEvent::HeartbeatRestored)?),
            PeerLifecycleEvent::Disconnect => {
                Ok(self.lifecycle.transition(PeerLifecycleEvent::Disconnect)?)
            }
            PeerLifecycleEvent::Rejoin => {
                Ok(self.lifecycle.transition(PeerLifecycleEvent::Rejoin)?)
            }
            PeerLifecycleEvent::StartConnect => Ok(self
                .lifecycle
                .transition(PeerLifecycleEvent::StartConnect)?),
        }
    }

    /// Discovers peers that advertise support for the provided gossip topic.
    pub fn discover(&self, topic: &str) -> Result<Vec<PeerDiscoveryRecord>, P2pTransportError> {
        self.require_active_state()?;
        self.transport.discover(&self.local_peer_id, topic)
    }

    /// Broadcasts payload to all discovered peers for a topic and returns delivery fan-out count.
    pub fn broadcast(&self, topic: &str, payload: &str) -> Result<usize, P2pTransportError> {
        self.require_active_state()?;
        validate_topic(topic)?;
        if payload.trim().is_empty() {
            return Err(P2pTransportError::EmptyPayload);
        }

        let discovered = self.transport.discover(&self.local_peer_id, topic)?;
        for peer in &discovered {
            let frame = PeerGossipFrame::new(topic, &self.local_peer_id, &peer.peer_id, payload)?;
            self.transport.send(frame)?;
        }
        Ok(discovered.len())
    }

    /// Drains all inbound frames currently queued for this peer.
    pub fn drain_inbox(&self) -> Result<Vec<PeerGossipFrame>, P2pTransportError> {
        self.transport.drain_inbox(&self.local_peer_id)
    }

    fn require_active_state(&self) -> Result<(), P2pTransportError> {
        match self.lifecycle.state() {
            PeerLifecycleState::Active => Ok(()),
            state => Err(P2pTransportError::InactivePeerLifecycleState(state)),
        }
    }
}
