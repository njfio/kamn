//! Deterministic peer discovery and gossip transport adapters for runtime integration.

use crate::config::NodeRole;
use crate::runtime::{
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};

/// Transport-level discovery metadata advertised by each active peer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerDiscoveryRecord {
    /// Unique transport peer identifier.
    pub peer_id: String,
    /// Runtime role advertised by the peer.
    pub role: NodeRole,
    /// Gossip topics advertised by this peer.
    pub gossip_topics: Vec<String>,
}

impl PeerDiscoveryRecord {
    /// Builds a validated discovery record with deterministic topic normalization.
    pub fn new(
        peer_id: &str,
        role: NodeRole,
        gossip_topics: Vec<String>,
    ) -> Result<Self, P2pTransportError> {
        validate_peer_id(peer_id)?;
        if gossip_topics.is_empty() {
            return Err(P2pTransportError::MissingGossipTopics);
        }

        let mut normalized = BTreeSet::new();
        for topic in gossip_topics {
            validate_topic(&topic)?;
            normalized.insert(topic.trim().to_owned());
        }

        Ok(Self {
            peer_id: peer_id.to_owned(),
            role,
            gossip_topics: normalized.into_iter().collect(),
        })
    }

    fn supports_topic(&self, topic: &str) -> bool {
        self.gossip_topics.iter().any(|value| value == topic)
    }
}

/// Deterministic gossip frame exchanged between peers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerGossipFrame {
    /// Gossip topic used for fan-out delivery.
    pub topic: String,
    /// Sender peer identifier.
    pub sender_peer_id: String,
    /// Recipient peer identifier.
    pub recipient_peer_id: String,
    /// Canonical payload body.
    pub payload: String,
}

impl PeerGossipFrame {
    /// Builds a validated gossip frame.
    pub fn new(
        topic: &str,
        sender_peer_id: &str,
        recipient_peer_id: &str,
        payload: &str,
    ) -> Result<Self, P2pTransportError> {
        validate_topic(topic)?;
        validate_peer_id(sender_peer_id)?;
        validate_peer_id(recipient_peer_id)?;
        if payload.trim().is_empty() {
            return Err(P2pTransportError::EmptyPayload);
        }
        Ok(Self {
            topic: topic.trim().to_owned(),
            sender_peer_id: sender_peer_id.to_owned(),
            recipient_peer_id: recipient_peer_id.to_owned(),
            payload: payload.to_owned(),
        })
    }
}

/// Transport adapter contract used by peer lifecycle coordinators.
pub trait PeerLifecycleTransport {
    /// Advertises a local peer for topic-based discovery.
    fn advertise(&self, record: PeerDiscoveryRecord) -> Result<(), P2pTransportError>;
    /// Discovers peers that support the requested topic.
    fn discover(
        &self,
        requester_peer_id: &str,
        topic: &str,
    ) -> Result<Vec<PeerDiscoveryRecord>, P2pTransportError>;
    /// Sends a gossip frame to the recipient inbox.
    fn send(&self, frame: PeerGossipFrame) -> Result<(), P2pTransportError>;
    /// Drains all queued frames for the local recipient.
    fn drain_inbox(
        &self,
        recipient_peer_id: &str,
    ) -> Result<Vec<PeerGossipFrame>, P2pTransportError>;
}

#[derive(Debug, Default)]
struct InMemoryPeerLifecycleTransportState {
    peers_by_id: BTreeMap<String, PeerDiscoveryRecord>,
    inbox_by_peer: BTreeMap<String, VecDeque<PeerGossipFrame>>,
}

/// Shared in-memory transport adapter used by deterministic tests and local smoke lanes.
#[derive(Debug, Clone, Default)]
pub struct InMemoryPeerLifecycleTransport {
    state: Arc<Mutex<InMemoryPeerLifecycleTransportState>>,
}

impl PeerLifecycleTransport for InMemoryPeerLifecycleTransport {
    fn advertise(&self, record: PeerDiscoveryRecord) -> Result<(), P2pTransportError> {
        let mut state = self.lock_state_mut()?;
        state
            .inbox_by_peer
            .entry(record.peer_id.clone())
            .or_insert_with(VecDeque::new);
        state.peers_by_id.insert(record.peer_id.clone(), record);
        Ok(())
    }

    fn discover(
        &self,
        requester_peer_id: &str,
        topic: &str,
    ) -> Result<Vec<PeerDiscoveryRecord>, P2pTransportError> {
        validate_peer_id(requester_peer_id)?;
        validate_topic(topic)?;
        let state = self.lock_state()?;

        Ok(state
            .peers_by_id
            .values()
            .filter(|record| record.peer_id != requester_peer_id && record.supports_topic(topic))
            .cloned()
            .collect())
    }

    fn send(&self, frame: PeerGossipFrame) -> Result<(), P2pTransportError> {
        let mut state = self.lock_state_mut()?;
        if !state.peers_by_id.contains_key(&frame.sender_peer_id) {
            return Err(P2pTransportError::UnknownSenderPeer(
                frame.sender_peer_id.clone(),
            ));
        }
        if !state.peers_by_id.contains_key(&frame.recipient_peer_id) {
            return Err(P2pTransportError::UnknownRecipientPeer(
                frame.recipient_peer_id.clone(),
            ));
        }
        state
            .inbox_by_peer
            .entry(frame.recipient_peer_id.clone())
            .or_insert_with(VecDeque::new)
            .push_back(frame);
        Ok(())
    }

    fn drain_inbox(
        &self,
        recipient_peer_id: &str,
    ) -> Result<Vec<PeerGossipFrame>, P2pTransportError> {
        validate_peer_id(recipient_peer_id)?;
        let mut state = self.lock_state_mut()?;
        let queue = state
            .inbox_by_peer
            .entry(recipient_peer_id.to_owned())
            .or_insert_with(VecDeque::new);
        Ok(queue.drain(..).collect())
    }
}

impl InMemoryPeerLifecycleTransport {
    fn lock_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, InMemoryPeerLifecycleTransportState>, P2pTransportError>
    {
        self.state
            .lock()
            .map_err(|_| P2pTransportError::StateUnavailable)
    }

    fn lock_state_mut(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, InMemoryPeerLifecycleTransportState>, P2pTransportError>
    {
        self.state
            .lock()
            .map_err(|_| P2pTransportError::StateUnavailable)
    }
}

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

/// Deterministic p2p discovery and gossip transport error variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum P2pTransportError {
    /// Peer identifier is empty.
    InvalidPeerId,
    /// Topic is empty or malformed for deterministic wire handling.
    InvalidTopic,
    /// Discovery record has no gossip topics.
    MissingGossipTopics,
    /// Gossip payload cannot be empty.
    EmptyPayload,
    /// Sender peer has not been advertised.
    UnknownSenderPeer(String),
    /// Recipient peer has not been advertised.
    UnknownRecipientPeer(String),
    /// Transport state lock is unavailable.
    StateUnavailable,
    /// Lifecycle state does not permit transport I/O.
    InactivePeerLifecycleState(PeerLifecycleState),
    /// Lifecycle transition error.
    Lifecycle(RuntimeLifecycleError),
}

impl From<RuntimeLifecycleError> for P2pTransportError {
    fn from(value: RuntimeLifecycleError) -> Self {
        Self::Lifecycle(value)
    }
}

impl Display for P2pTransportError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPeerId => write!(f, "p2p peer id cannot be empty"),
            Self::InvalidTopic => write!(f, "p2p topic cannot be empty or contain wire delimiters"),
            Self::MissingGossipTopics => write!(f, "p2p discovery topics cannot be empty"),
            Self::EmptyPayload => write!(f, "p2p gossip payload cannot be empty"),
            Self::UnknownSenderPeer(peer_id) => {
                write!(f, "p2p sender peer is not advertised: {peer_id}")
            }
            Self::UnknownRecipientPeer(peer_id) => {
                write!(f, "p2p recipient peer is not advertised: {peer_id}")
            }
            Self::StateUnavailable => write!(f, "p2p in-memory transport state is unavailable"),
            Self::InactivePeerLifecycleState(state) => {
                write!(
                    f,
                    "p2p transport requires active lifecycle state, found {state:?}"
                )
            }
            Self::Lifecycle(error) => write!(f, "{error}"),
        }
    }
}

impl Error for P2pTransportError {}

fn validate_peer_id(peer_id: &str) -> Result<(), P2pTransportError> {
    if peer_id.trim().is_empty() {
        return Err(P2pTransportError::InvalidPeerId);
    }
    Ok(())
}

fn validate_topic(topic: &str) -> Result<(), P2pTransportError> {
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

#[cfg(test)]
mod tests {
    use super::{
        InMemoryPeerLifecycleTransport, P2pTransportError, PeerGossipFrame, PeerLifecycleTransport,
    };
    use crate::config::NodeRole;
    use crate::p2p_transport::{PeerDiscoveryRecord, PeerLifecycleTransportCoordinator};
    use crate::runtime::PeerLifecycleState;

    #[test]
    fn transport_discovery_filters_by_topic_and_excludes_requester() {
        let transport = InMemoryPeerLifecycleTransport::default();
        transport
            .advertise(
                PeerDiscoveryRecord::new(
                    "peer-a",
                    NodeRole::Processor,
                    vec!["messages".to_owned(), "blocks".to_owned()],
                )
                .expect("record should be valid"),
            )
            .expect("advertise peer-a");
        transport
            .advertise(
                PeerDiscoveryRecord::new("peer-b", NodeRole::Listener, vec!["messages".to_owned()])
                    .expect("record should be valid"),
            )
            .expect("advertise peer-b");
        transport
            .advertise(
                PeerDiscoveryRecord::new("peer-c", NodeRole::Approver, vec!["blocks".to_owned()])
                    .expect("record should be valid"),
            )
            .expect("advertise peer-c");

        let discovered = transport
            .discover("peer-a", "messages")
            .expect("discovery should pass");
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].peer_id, "peer-b");
    }

    #[test]
    fn coordinator_connect_and_advertise_transitions_to_active_state() {
        let transport = InMemoryPeerLifecycleTransport::default();
        let mut coordinator = PeerLifecycleTransportCoordinator::new(
            "peer-processor",
            NodeRole::Processor,
            transport,
        )
        .expect("coordinator should initialize");
        let state = coordinator
            .connect_and_advertise(vec!["messages".to_owned()])
            .expect("connect and advertise should pass");
        assert_eq!(state, PeerLifecycleState::Active);
        assert_eq!(coordinator.lifecycle_state(), PeerLifecycleState::Active);
    }

    #[test]
    fn transport_send_rejects_unknown_recipient() {
        let transport = InMemoryPeerLifecycleTransport::default();
        transport
            .advertise(
                PeerDiscoveryRecord::new(
                    "peer-sender",
                    NodeRole::Processor,
                    vec!["messages".to_owned()],
                )
                .expect("record should be valid"),
            )
            .expect("advertise sender");

        let frame = PeerGossipFrame::new("messages", "peer-sender", "peer-missing", "tx:001")
            .expect("frame should be valid");
        let result = transport.send(frame);
        assert_eq!(
            result,
            Err(P2pTransportError::UnknownRecipientPeer(
                "peer-missing".to_owned()
            ))
        );
    }

    #[test]
    fn regression_coordinator_requires_active_state_for_broadcast() {
        // Regression: #2922
        let transport = InMemoryPeerLifecycleTransport::default();
        let coordinator = PeerLifecycleTransportCoordinator::new(
            "peer-processor",
            NodeRole::Processor,
            transport,
        )
        .expect("coordinator should initialize");

        assert_eq!(
            coordinator.broadcast("messages", "tx:001"),
            Err(P2pTransportError::InactivePeerLifecycleState(
                PeerLifecycleState::Disconnected
            ))
        );
    }
}
