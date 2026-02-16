use crate::config::NodeRole;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, OnceLock};

use super::validation::{validate_peer_id, validate_topic};
use super::P2pTransportError;

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

    pub(super) fn supports_topic(&self, topic: &str) -> bool {
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

    pub(super) fn encode_frame(frame: &PeerGossipFrame) -> Vec<u8> {
        format!(
            "{}\n{}\n{}\n{}",
            frame.topic, frame.sender_peer_id, frame.recipient_peer_id, frame.payload
        )
        .into_bytes()
    }

    pub(super) fn decode_frame(payload: &[u8]) -> Result<PeerGossipFrame, P2pTransportError> {
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
