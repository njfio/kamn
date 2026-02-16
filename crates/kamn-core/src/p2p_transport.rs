//! Deterministic peer discovery and gossip transport adapters for runtime integration.

use crate::config::{NodeConfig, NodeRole};
use crate::runtime::{
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError,
    RuntimeTransportProfile,
};
#[cfg(feature = "libp2p-live-transport")]
use libp2p::{gossipsub, identify};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, OnceLock};

const LIBP2P_SWARM_BEHAVIOR_COMPONENTS: [&str; 6] =
    ["tcp", "noise", "yamux", "identify", "kad", "gossipsub"];
const LIBP2P_IDENTIFY_PROTOCOL_ID: &str = "/kamn/libp2p-live/1.0.0";
const LIBP2P_TOPIC_NAMESPACE: &str = "kamn/v1/";
const LIBP2P_RUNTIME_EVENT_SCHEMA_MARKER: &str = "kamn.libp2p.runtime-event.v1";

#[cfg(feature = "libp2p-live-transport")]
#[derive(libp2p::swarm::NetworkBehaviour)]
struct Libp2pDeterministicRuntimeBehaviour {
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
}

/// Failure classes produced while normalizing deterministic libp2p runtime events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Libp2pBehaviorFailureClass {
    /// Runtime publish call was rejected by behavior-level policy.
    PublishRejected,
    /// Runtime observed malformed or disallowed topic metadata.
    InvalidTopic,
    /// Runtime observed malformed peer metadata.
    InvalidPeerEvent,
    /// Runtime adapter connect control channel closed unexpectedly.
    RuntimeConnectChannelClosed,
    /// Runtime adapter discover control channel closed unexpectedly.
    RuntimeDiscoverChannelClosed,
    /// Runtime adapter publish control channel closed unexpectedly.
    RuntimePublishChannelClosed,
    /// Runtime adapter receive control channel closed unexpectedly.
    RuntimeReceiveChannelClosed,
    /// Runtime adapter runtime-event drain control channel closed unexpectedly.
    RuntimeEventDrainChannelClosed,
    /// Runtime frame send failed because sender peer was unknown.
    UnknownSenderPeer,
    /// Runtime frame send failed because recipient peer was unknown.
    UnknownRecipientPeer,
}

impl Libp2pBehaviorFailureClass {
    /// Returns deterministic reason code used by runtime policy checks.
    pub fn reason_code(self) -> &'static str {
        match self {
            Self::PublishRejected => "p2p_libp2p_publish_rejected",
            Self::InvalidTopic => "p2p_transport_invalid_topic",
            Self::InvalidPeerEvent => "p2p_libp2p_event_invalid_peer_event",
            Self::RuntimeConnectChannelClosed => "p2p_libp2p_runtime_connect_channel_closed",
            Self::RuntimeDiscoverChannelClosed => "p2p_libp2p_runtime_discover_channel_closed",
            Self::RuntimePublishChannelClosed => "p2p_libp2p_runtime_publish_channel_closed",
            Self::RuntimeReceiveChannelClosed => "p2p_libp2p_runtime_receive_channel_closed",
            Self::RuntimeEventDrainChannelClosed => "p2p_libp2p_runtime_event_drain_channel_closed",
            Self::UnknownSenderPeer => "p2p_transport_unknown_sender_peer",
            Self::UnknownRecipientPeer => "p2p_transport_unknown_recipient_peer",
        }
    }
}

/// Runtime event kinds emitted by deterministic libp2p adapter normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Libp2pRuntimeEventKind {
    /// Local peer advertised discovery metadata.
    PeerAdvertised,
    /// Discovery lookup returned one peer for requested topic.
    PeerDiscovered,
    /// Gossip payload published to one recipient.
    GossipPublished,
    /// Gossip payload accepted for one recipient inbox.
    GossipReceived,
    /// Behavior-level failure was emitted.
    BehaviorFailure,
}

/// Deterministic libp2p runtime event schema consumed by runtime adapter policy checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Libp2pRuntimeEvent {
    schema_marker: &'static str,
    kind: Libp2pRuntimeEventKind,
    peer_id: Option<String>,
    topic_id: Option<String>,
    payload: Option<String>,
    reason_code: &'static str,
}

impl Libp2pRuntimeEvent {
    fn new(
        kind: Libp2pRuntimeEventKind,
        peer_id: Option<String>,
        topic_id: Option<String>,
        payload: Option<String>,
        reason_code: &'static str,
    ) -> Self {
        Self {
            schema_marker: LIBP2P_RUNTIME_EVENT_SCHEMA_MARKER,
            kind,
            peer_id,
            topic_id,
            payload,
            reason_code,
        }
    }

    /// Builds normalized event for one successful peer advertisement.
    pub fn peer_advertised(peer_id: &str) -> Result<Self, P2pTransportError> {
        validate_peer_id(peer_id)?;
        Ok(Self::new(
            Libp2pRuntimeEventKind::PeerAdvertised,
            Some(peer_id.to_owned()),
            None,
            None,
            "p2p_libp2p_event_peer_advertised",
        ))
    }

    /// Builds normalized event for one discovered peer and topic.
    pub fn peer_discovered(peer_id: &str, topic: &str) -> Result<Self, P2pTransportError> {
        validate_peer_id(peer_id)?;
        let topic_id = canonical_libp2p_topic_id(topic)?;
        Ok(Self::new(
            Libp2pRuntimeEventKind::PeerDiscovered,
            Some(peer_id.to_owned()),
            Some(topic_id),
            None,
            "p2p_libp2p_event_peer_discovered",
        ))
    }

    /// Builds normalized event for one successful gossip publish operation.
    pub fn gossip_published(
        peer_id: &str,
        topic: &str,
        payload: &str,
    ) -> Result<Self, P2pTransportError> {
        validate_peer_id(peer_id)?;
        let topic_id = canonical_libp2p_topic_id(topic)?;
        if payload.trim().is_empty() {
            return Err(P2pTransportError::EmptyPayload);
        }
        Ok(Self::new(
            Libp2pRuntimeEventKind::GossipPublished,
            Some(peer_id.to_owned()),
            Some(topic_id),
            Some(payload.to_owned()),
            "p2p_libp2p_event_gossip_published",
        ))
    }

    /// Builds normalized event for one accepted gossip message delivery.
    pub fn gossip_received(
        peer_id: &str,
        topic: &str,
        payload: &str,
    ) -> Result<Self, P2pTransportError> {
        validate_peer_id(peer_id)?;
        let topic_id = canonical_libp2p_topic_id(topic)?;
        if payload.trim().is_empty() {
            return Err(P2pTransportError::EmptyPayload);
        }
        Ok(Self::new(
            Libp2pRuntimeEventKind::GossipReceived,
            Some(peer_id.to_owned()),
            Some(topic_id),
            Some(payload.to_owned()),
            "p2p_libp2p_event_gossip_received",
        ))
    }

    /// Builds normalized event for one behavior-level failure.
    pub fn behavior_failure(
        class: Libp2pBehaviorFailureClass,
        peer_id: Option<&str>,
        topic: Option<&str>,
    ) -> Result<Self, P2pTransportError> {
        let peer_id = match peer_id {
            Some(value) => {
                validate_peer_id(value)?;
                Some(value.to_owned())
            }
            None => None,
        };
        let topic_id = match topic {
            Some(value) => Some(canonical_libp2p_topic_id(value)?),
            None => None,
        };
        Ok(Self::new(
            Libp2pRuntimeEventKind::BehaviorFailure,
            peer_id,
            topic_id,
            None,
            class.reason_code(),
        ))
    }

    /// Returns deterministic schema marker for this event payload.
    pub fn schema_marker(&self) -> &'static str {
        self.schema_marker
    }

    /// Returns event kind.
    pub fn kind(&self) -> Libp2pRuntimeEventKind {
        self.kind
    }

    /// Returns optional peer identifier.
    pub fn peer_id(&self) -> Option<&str> {
        self.peer_id.as_deref()
    }

    /// Returns optional canonical topic identifier.
    pub fn topic_id(&self) -> Option<&str> {
        self.topic_id.as_deref()
    }

    /// Returns optional payload body.
    pub fn payload(&self) -> Option<&str> {
        self.payload.as_deref()
    }

    /// Returns deterministic reason code.
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}

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

#[derive(Debug, Default)]
struct UdpPeerLifecycleTransportState {
    peers_by_id: BTreeMap<String, PeerDiscoveryRecord>,
    socket_addr_by_peer: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
/// UDP socket-backed transport adapter for live local convergence drill execution.
pub struct UdpPeerLifecycleTransport {
    network_id: String,
    local_peer_id: String,
    socket: Arc<UdpSocket>,
    state: Arc<Mutex<UdpPeerLifecycleTransportState>>,
}

impl UdpPeerLifecycleTransport {
    /// Binds a UDP socket-backed transport adapter to one local peer id.
    pub fn bind(
        network_id: &str,
        local_peer_id: &str,
        bind_address: &str,
    ) -> Result<Self, P2pTransportError> {
        if network_id.trim().is_empty() {
            return Err(P2pTransportError::InvalidLiveSocketNetworkId);
        }
        validate_peer_id(local_peer_id)?;
        if bind_address.trim().is_empty() {
            return Err(P2pTransportError::InvalidSocketBindAddress);
        }

        let socket =
            UdpSocket::bind(bind_address).map_err(|_| P2pTransportError::LiveSocketBindFailed)?;
        socket
            .set_nonblocking(true)
            .map_err(|_| P2pTransportError::LiveSocketBindFailed)?;
        let state = resolve_udp_live_transport_state(network_id)?;

        Ok(Self {
            network_id: network_id.to_owned(),
            local_peer_id: local_peer_id.to_owned(),
            socket: Arc::new(socket),
            state,
        })
    }

    /// Binds a UDP socket-backed transport adapter to an ephemeral local port.
    pub fn bind_ephemeral(
        network_id: &str,
        local_peer_id: &str,
    ) -> Result<Self, P2pTransportError> {
        Self::bind(network_id, local_peer_id, "127.0.0.1:0")
    }

    /// Returns the deterministic network identifier for this live socket transport.
    pub fn network_id(&self) -> &str {
        &self.network_id
    }

    fn lock_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, UdpPeerLifecycleTransportState>, P2pTransportError> {
        self.state
            .lock()
            .map_err(|_| P2pTransportError::StateUnavailable)
    }

    fn encode_frame(frame: &PeerGossipFrame) -> Vec<u8> {
        format!(
            "{}\n{}\n{}\n{}",
            frame.topic, frame.sender_peer_id, frame.recipient_peer_id, frame.payload
        )
        .into_bytes()
    }

    fn decode_frame(payload: &[u8]) -> Result<PeerGossipFrame, P2pTransportError> {
        let raw = std::str::from_utf8(payload)
            .map_err(|_| P2pTransportError::LiveSocketFrameMalformed)?;
        let mut parts = raw.splitn(4, '\n');
        let topic = parts
            .next()
            .ok_or(P2pTransportError::LiveSocketFrameMalformed)?;
        let sender_peer_id = parts
            .next()
            .ok_or(P2pTransportError::LiveSocketFrameMalformed)?;
        let recipient_peer_id = parts
            .next()
            .ok_or(P2pTransportError::LiveSocketFrameMalformed)?;
        let frame_payload = parts
            .next()
            .ok_or(P2pTransportError::LiveSocketFrameMalformed)?;
        PeerGossipFrame::new(topic, sender_peer_id, recipient_peer_id, frame_payload)
    }
}

impl PeerLifecycleTransport for UdpPeerLifecycleTransport {
    fn advertise(&self, record: PeerDiscoveryRecord) -> Result<(), P2pTransportError> {
        let socket_address = self
            .socket
            .local_addr()
            .map_err(|_| P2pTransportError::LiveSocketLocalAddressUnavailable)?
            .to_string();
        let mut state = self.lock_state()?;
        state
            .socket_addr_by_peer
            .insert(record.peer_id.clone(), socket_address);
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
        let recipient_socket_address = {
            let state = self.lock_state()?;
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
                .socket_addr_by_peer
                .get(&frame.recipient_peer_id)
                .cloned()
                .ok_or_else(|| {
                    P2pTransportError::UnknownRecipientPeer(frame.recipient_peer_id.clone())
                })?
        };

        let encoded = Self::encode_frame(&frame);
        self.socket
            .send_to(encoded.as_slice(), recipient_socket_address.as_str())
            .map_err(|_| P2pTransportError::LiveSocketSendFailed)?;
        Ok(())
    }

    fn drain_inbox(
        &self,
        recipient_peer_id: &str,
    ) -> Result<Vec<PeerGossipFrame>, P2pTransportError> {
        validate_peer_id(recipient_peer_id)?;
        if recipient_peer_id != self.local_peer_id {
            return Err(P2pTransportError::UnknownRecipientPeer(
                recipient_peer_id.to_owned(),
            ));
        }

        let mut frames = Vec::new();
        let mut buffer = [0u8; 32 * 1024];
        loop {
            match self.socket.recv_from(&mut buffer) {
                Ok((payload_size, _source)) => {
                    let frame = Self::decode_frame(&buffer[..payload_size])?;
                    if frame.recipient_peer_id == recipient_peer_id {
                        frames.push(frame);
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(_) => return Err(P2pTransportError::LiveSocketReceiveFailed),
            }
        }
        Ok(frames)
    }
}

fn udp_live_transport_registry(
) -> &'static Mutex<BTreeMap<String, Arc<Mutex<UdpPeerLifecycleTransportState>>>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, Arc<Mutex<UdpPeerLifecycleTransportState>>>>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn resolve_udp_live_transport_state(
    network_id: &str,
) -> Result<Arc<Mutex<UdpPeerLifecycleTransportState>>, P2pTransportError> {
    let mut registry = udp_live_transport_registry()
        .lock()
        .map_err(|_| P2pTransportError::StateUnavailable)?;
    Ok(registry
        .entry(network_id.to_owned())
        .or_insert_with(|| Arc::new(Mutex::new(UdpPeerLifecycleTransportState::default())))
        .clone())
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

mod p2p_transport_live;

pub use p2p_transport_live::*;

#[cfg(all(test, feature = "libp2p-live-transport"))]
use p2p_transport_live::{
    runtime_channel_closed_behavior_failure_class, Libp2pNativeRuntimeAdapterLoop,
};
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
    /// Kademlia discovery bootstrap requires at least one seed peer.
    MissingKademliaBootstrapSeeds,
    /// Swarm listen address is empty or malformed for deterministic multiaddr handling.
    InvalidSwarmListenAddress,
    /// Swarm bootstrap peer address is malformed for deterministic multiaddr handling.
    InvalidSwarmBootstrapPeerAddress(String),
    /// Swarm harness tick budget must be positive.
    InvalidSwarmHarnessTickBudget,
    /// Reconnect retry budget must be positive.
    InvalidReconnectRetryBudget,
    /// Reconnect backoff window is invalid.
    InvalidReconnectBackoffWindow,
    /// Swarm composition requested while gossip transport is disabled.
    GossipTransportDisabled,
    /// Live socket network id is empty.
    InvalidLiveSocketNetworkId,
    /// Live socket bind address is empty or malformed.
    InvalidSocketBindAddress,
    /// Live socket bind operation failed.
    LiveSocketBindFailed,
    /// Live socket local address lookup failed.
    LiveSocketLocalAddressUnavailable,
    /// Live socket datagram send operation failed.
    LiveSocketSendFailed,
    /// Live socket datagram receive operation failed.
    LiveSocketReceiveFailed,
    /// Live socket datagram payload is malformed.
    LiveSocketFrameMalformed,
    /// Libp2p runtime config validation failed in native mode.
    Libp2pRuntimeConfigInvalid,
    /// Libp2p native runtime adapter command channel closed unexpectedly.
    Libp2pRuntimeAdapterChannelClosed(Libp2pRuntimeAdapterOperation),
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

impl P2pTransportError {
    /// Returns deterministic reason code for policy and regression guard checks.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidPeerId => "p2p_transport_invalid_peer_id",
            Self::InvalidTopic => "p2p_transport_invalid_topic",
            Self::MissingGossipTopics => "p2p_transport_missing_gossip_topics",
            Self::EmptyPayload => "p2p_transport_empty_payload",
            Self::UnknownSenderPeer(_) => "p2p_transport_unknown_sender_peer",
            Self::UnknownRecipientPeer(_) => "p2p_transport_unknown_recipient_peer",
            Self::MissingKademliaBootstrapSeeds => "p2p_transport_missing_kademlia_seeds",
            Self::InvalidSwarmListenAddress => "p2p_transport_invalid_swarm_listen_address",
            Self::InvalidSwarmBootstrapPeerAddress(_) => {
                "p2p_transport_invalid_swarm_bootstrap_peer_address"
            }
            Self::InvalidSwarmHarnessTickBudget => "p2p_transport_invalid_harness_tick_budget",
            Self::InvalidReconnectRetryBudget => "p2p_transport_invalid_reconnect_retry_budget",
            Self::InvalidReconnectBackoffWindow => "p2p_transport_invalid_reconnect_backoff_window",
            Self::GossipTransportDisabled => "p2p_transport_gossip_disabled",
            Self::InvalidLiveSocketNetworkId => "p2p_transport_live_socket_network_id_invalid",
            Self::InvalidSocketBindAddress => "p2p_transport_live_socket_bind_address_invalid",
            Self::LiveSocketBindFailed => "p2p_transport_live_socket_bind_failed",
            Self::LiveSocketLocalAddressUnavailable => {
                "p2p_transport_live_socket_local_address_unavailable"
            }
            Self::LiveSocketSendFailed => "p2p_transport_live_socket_send_failed",
            Self::LiveSocketReceiveFailed => "p2p_transport_live_socket_receive_failed",
            Self::LiveSocketFrameMalformed => "p2p_transport_live_socket_frame_malformed",
            Self::Libp2pRuntimeConfigInvalid => "p2p_transport_libp2p_runtime_config_invalid",
            Self::Libp2pRuntimeAdapterChannelClosed(operation) => {
                operation.channel_closed_reason_code()
            }
            Self::StateUnavailable => "p2p_transport_state_unavailable",
            Self::InactivePeerLifecycleState(_) => "p2p_transport_inactive_lifecycle_state",
            Self::Lifecycle(error) => error.reason_code(),
        }
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
            Self::MissingKademliaBootstrapSeeds => {
                write!(f, "p2p kademlia bootstrap seed set cannot be empty")
            }
            Self::InvalidSwarmListenAddress => {
                write!(f, "p2p swarm listen address must be a tcp multiaddr")
            }
            Self::InvalidSwarmBootstrapPeerAddress(value) => {
                write!(f, "p2p swarm bootstrap peer address is invalid: {value}")
            }
            Self::InvalidSwarmHarnessTickBudget => {
                write!(f, "p2p swarm harness tick budget must be positive")
            }
            Self::InvalidReconnectRetryBudget => {
                write!(f, "p2p reconnect retry budget must be positive")
            }
            Self::InvalidReconnectBackoffWindow => write!(
                f,
                "p2p reconnect backoff window is invalid (base/max must be positive and base <= max)"
            ),
            Self::GossipTransportDisabled => {
                write!(
                    f,
                    "p2p swarm composition requires gossip transport to be enabled"
                )
            }
            Self::InvalidLiveSocketNetworkId => {
                write!(f, "p2p live socket network id cannot be empty")
            }
            Self::InvalidSocketBindAddress => {
                write!(f, "p2p live socket bind address is invalid")
            }
            Self::LiveSocketBindFailed => write!(f, "p2p live socket bind failed"),
            Self::LiveSocketLocalAddressUnavailable => {
                write!(f, "p2p live socket local address lookup failed")
            }
            Self::LiveSocketSendFailed => write!(f, "p2p live socket send failed"),
            Self::LiveSocketReceiveFailed => write!(f, "p2p live socket receive failed"),
            Self::LiveSocketFrameMalformed => write!(f, "p2p live socket frame is malformed"),
            Self::Libp2pRuntimeConfigInvalid => {
                write!(f, "p2p libp2p native runtime config is invalid")
            }
            Self::Libp2pRuntimeAdapterChannelClosed(operation) => write!(
                f,
                "p2p libp2p runtime adapter channel closed during {} operation",
                operation.as_str()
            ),
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

fn validate_swarm_listen_address(listen_address: &str) -> Result<(), P2pTransportError> {
    if is_supported_swarm_multiaddr(listen_address) {
        return Ok(());
    }
    Err(P2pTransportError::InvalidSwarmListenAddress)
}

fn validate_swarm_bootstrap_peer_address(address: &str) -> Result<(), P2pTransportError> {
    if is_supported_swarm_multiaddr(address) {
        return Ok(());
    }
    Err(P2pTransportError::InvalidSwarmBootstrapPeerAddress(
        address.to_owned(),
    ))
}

fn is_supported_swarm_multiaddr(value: &str) -> bool {
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

#[cfg(test)]
mod tests {
    #[cfg(feature = "libp2p-live-transport")]
    use super::Libp2pRuntimeEventKind;
    use super::{
        InMemoryPeerLifecycleTransport, Libp2pBehaviorFailureClass, Libp2pRuntimeAdapterOperation,
        Libp2pRuntimeEvent, P2pTransportError, PeerGossipFrame, PeerLifecycleTransport,
    };
    use crate::config::NodeRole;
    use crate::p2p_transport::{PeerDiscoveryRecord, PeerLifecycleTransportCoordinator};
    use crate::runtime::{PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError};
    #[cfg(feature = "libp2p-live-transport")]
    use std::sync::{Arc, Mutex};

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

    #[test]
    fn transport_error_reason_code_remains_deterministic() {
        assert_eq!(
            P2pTransportError::UnknownRecipientPeer("peer-z".to_owned()).reason_code(),
            "p2p_transport_unknown_recipient_peer"
        );
        assert_eq!(
            P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                Libp2pRuntimeAdapterOperation::Connect,
            )
            .reason_code(),
            "p2p_libp2p_runtime_connect_channel_closed"
        );
        assert_eq!(
            P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                Libp2pRuntimeAdapterOperation::Publish,
            )
            .reason_code(),
            "p2p_libp2p_runtime_publish_channel_closed"
        );
        assert_eq!(
            P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                Libp2pRuntimeAdapterOperation::Discover,
            )
            .reason_code(),
            "p2p_libp2p_runtime_discover_channel_closed"
        );
        assert_eq!(
            P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                Libp2pRuntimeAdapterOperation::Receive,
            )
            .reason_code(),
            "p2p_libp2p_runtime_receive_channel_closed"
        );
        assert_eq!(
            P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                Libp2pRuntimeAdapterOperation::EventDrain,
            )
            .reason_code(),
            "p2p_libp2p_runtime_event_drain_channel_closed"
        );
        assert_eq!(
            P2pTransportError::Lifecycle(RuntimeLifecycleError::InvalidTransition {
                from: PeerLifecycleState::Disconnected,
                event: PeerLifecycleEvent::HeartbeatRestored,
            })
            .reason_code(),
            "runtime_peer_transition_invalid"
        );
    }

    #[test]
    fn runtime_behavior_failure_reason_code_for_native_channel_close_is_operation_scoped() {
        assert_eq!(
            Libp2pRuntimeEvent::behavior_failure(
                Libp2pBehaviorFailureClass::RuntimeConnectChannelClosed,
                None,
                None,
            )
            .expect("behavior failure should build")
            .reason_code(),
            "p2p_libp2p_runtime_connect_channel_closed"
        );
        assert_eq!(
            Libp2pRuntimeEvent::behavior_failure(
                Libp2pBehaviorFailureClass::RuntimePublishChannelClosed,
                None,
                None,
            )
            .expect("behavior failure should build")
            .reason_code(),
            "p2p_libp2p_runtime_publish_channel_closed"
        );
    }

    #[cfg(feature = "libp2p-live-transport")]
    #[test]
    fn native_runtime_loop_channel_close_records_behavior_failure_event() {
        let (runtime_loop, state) = build_closed_native_runtime_loop();

        let error = runtime_loop
            .discover("peer-runtime-loop", "messages")
            .expect_err("closed bridge should fail");
        assert_eq!(
            error,
            P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                Libp2pRuntimeAdapterOperation::Discover,
            )
        );

        let events = state
            .lock()
            .expect("state lock should succeed")
            .runtime_events
            .drain(..)
            .collect::<Vec<Libp2pRuntimeEvent>>();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind(), Libp2pRuntimeEventKind::BehaviorFailure);
        assert_eq!(
            events[0].reason_code(),
            "p2p_libp2p_runtime_discover_channel_closed"
        );
    }

    #[cfg(feature = "libp2p-live-transport")]
    fn build_closed_native_runtime_loop() -> (
        super::Libp2pNativeRuntimeAdapterLoop,
        Arc<Mutex<super::Libp2pLiveDataPlaneState>>,
    ) {
        let state = Arc::new(Mutex::new(super::Libp2pLiveDataPlaneState::default()));
        let (command_tx, command_rx) = std::sync::mpsc::channel();
        drop(command_rx);
        (
            super::Libp2pNativeRuntimeAdapterLoop {
                command_tx,
                state: state.clone(),
            },
            state,
        )
    }

    #[cfg(feature = "libp2p-live-transport")]
    #[test]
    fn native_runtime_loop_channel_close_operation_mapping_is_deterministic() {
        {
            let (runtime_loop, state) = build_closed_native_runtime_loop();
            let error = runtime_loop
                .advertise(
                    PeerDiscoveryRecord::new(
                        "peer-connect-op",
                        NodeRole::Processor,
                        vec!["messages".to_owned()],
                    )
                    .expect("record should build"),
                )
                .expect_err("closed bridge should fail");
            assert_eq!(
                error,
                P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                    Libp2pRuntimeAdapterOperation::Connect,
                )
            );
            let events = state
                .lock()
                .expect("state lock should succeed")
                .runtime_events
                .drain(..)
                .collect::<Vec<Libp2pRuntimeEvent>>();
            assert_eq!(events.len(), 1);
            assert_eq!(
                events[0].reason_code(),
                "p2p_libp2p_runtime_connect_channel_closed"
            );
        }

        {
            let (runtime_loop, state) = build_closed_native_runtime_loop();
            let error = runtime_loop
                .discover("peer-discover-op", "messages")
                .expect_err("closed bridge should fail");
            assert_eq!(
                error,
                P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                    Libp2pRuntimeAdapterOperation::Discover,
                )
            );
            let events = state
                .lock()
                .expect("state lock should succeed")
                .runtime_events
                .drain(..)
                .collect::<Vec<Libp2pRuntimeEvent>>();
            assert_eq!(events.len(), 1);
            assert_eq!(
                events[0].reason_code(),
                "p2p_libp2p_runtime_discover_channel_closed"
            );
        }

        {
            let (runtime_loop, state) = build_closed_native_runtime_loop();
            let error = runtime_loop
                .send(
                    PeerGossipFrame::new(
                        "messages",
                        "peer-publish-op",
                        "peer-recipient-op",
                        "tx-runtime-op",
                    )
                    .expect("frame should build"),
                )
                .expect_err("closed bridge should fail");
            assert_eq!(
                error,
                P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                    Libp2pRuntimeAdapterOperation::Publish,
                )
            );
            let events = state
                .lock()
                .expect("state lock should succeed")
                .runtime_events
                .drain(..)
                .collect::<Vec<Libp2pRuntimeEvent>>();
            assert_eq!(events.len(), 1);
            assert_eq!(
                events[0].reason_code(),
                "p2p_libp2p_runtime_publish_channel_closed"
            );
        }

        {
            let (runtime_loop, state) = build_closed_native_runtime_loop();
            let error = runtime_loop
                .drain_inbox("peer-receive-op")
                .expect_err("closed bridge should fail");
            assert_eq!(
                error,
                P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                    Libp2pRuntimeAdapterOperation::Receive,
                )
            );
            let events = state
                .lock()
                .expect("state lock should succeed")
                .runtime_events
                .drain(..)
                .collect::<Vec<Libp2pRuntimeEvent>>();
            assert_eq!(events.len(), 1);
            assert_eq!(
                events[0].reason_code(),
                "p2p_libp2p_runtime_receive_channel_closed"
            );
        }

        {
            let (runtime_loop, state) = build_closed_native_runtime_loop();
            let error = runtime_loop
                .drain_runtime_events()
                .expect_err("closed bridge should fail");
            assert_eq!(
                error,
                P2pTransportError::Libp2pRuntimeAdapterChannelClosed(
                    Libp2pRuntimeAdapterOperation::EventDrain,
                )
            );
            let events = state
                .lock()
                .expect("state lock should succeed")
                .runtime_events
                .drain(..)
                .collect::<Vec<Libp2pRuntimeEvent>>();
            assert_eq!(events.len(), 1);
            assert_eq!(
                events[0].reason_code(),
                "p2p_libp2p_runtime_event_drain_channel_closed"
            );
        }
    }
}
