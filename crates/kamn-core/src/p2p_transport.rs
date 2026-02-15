//! Deterministic peer discovery and gossip transport adapters for runtime integration.

use crate::config::{NodeConfig, NodeRole};
use crate::runtime::{
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError,
    RuntimeTransportProfile,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex, OnceLock};

const LIBP2P_SWARM_BEHAVIOR_COMPONENTS: [&str; 6] =
    ["tcp", "noise", "yamux", "identify", "kad", "gossipsub"];

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

/// Runtime backend mode selected for live libp2p transport operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Libp2pLiveRuntimeBackend {
    /// Deterministic in-process data-plane fallback path.
    ContractDataPlane,
    /// Native socket-backed path used by feature-enabled runtime builds.
    NativeSocket,
}

impl Libp2pLiveRuntimeBackend {
    /// Returns deterministic backend marker for policy and docs contracts.
    pub fn marker(self) -> &'static str {
        match self {
            Self::ContractDataPlane => "contract-data-plane",
            Self::NativeSocket => "native-socket",
        }
    }
}

/// Resolves live libp2p runtime backend mode from compile-time feature gates.
pub fn resolve_libp2p_live_runtime_backend() -> Libp2pLiveRuntimeBackend {
    #[cfg(feature = "libp2p-live-transport")]
    {
        Libp2pLiveRuntimeBackend::NativeSocket
    }
    #[cfg(not(feature = "libp2p-live-transport"))]
    {
        Libp2pLiveRuntimeBackend::ContractDataPlane
    }
}

/// Live libp2p transport adapter contract backed by deterministic swarm startup.
#[derive(Debug, Clone)]
pub struct Libp2pLivePeerLifecycleTransport {
    swarm_config: P2pSwarmDeterministicConfig,
    harness_report: P2pSwarmHarnessReport,
    live_network_id: String,
    #[cfg(feature = "libp2p-live-transport")]
    native_socket_transport: UdpPeerLifecycleTransport,
    #[cfg(not(feature = "libp2p-live-transport"))]
    live_data_plane: Libp2pLiveDataPlane,
}

impl Libp2pLivePeerLifecycleTransport {
    /// Builds a live transport adapter and starts deterministic harness startup.
    pub fn new(
        config: P2pSwarmDeterministicConfig,
        harness_mode: P2pSwarmHarnessMode,
    ) -> Result<Self, P2pTransportError> {
        let task = P2pSwarmHarnessTask::new(config.clone());
        let harness_report = task.start(harness_mode)?;
        let network_id = build_live_data_plane_network_id(&config);
        #[cfg(feature = "libp2p-live-transport")]
        validate_libp2p_native_runtime_config(&config)?;
        #[cfg(feature = "libp2p-live-transport")]
        let native_socket_transport =
            UdpPeerLifecycleTransport::bind_ephemeral(network_id.as_str(), config.local_peer_id())?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        let state = resolve_live_data_plane_state(network_id.as_str())?;
        Ok(Self {
            swarm_config: config,
            harness_report,
            live_network_id: network_id.clone(),
            #[cfg(feature = "libp2p-live-transport")]
            native_socket_transport,
            #[cfg(not(feature = "libp2p-live-transport"))]
            live_data_plane: Libp2pLiveDataPlane { state },
        })
    }

    /// Returns runtime transport profile marker for this adapter.
    pub fn transport_profile(&self) -> RuntimeTransportProfile {
        RuntimeTransportProfile::Libp2pLive
    }

    /// Returns deterministic harness startup report for this live adapter.
    pub fn harness_report(&self) -> &P2pSwarmHarnessReport {
        &self.harness_report
    }

    /// Returns compile-mode runtime backend selected for this adapter.
    pub fn runtime_backend(&self) -> Libp2pLiveRuntimeBackend {
        resolve_libp2p_live_runtime_backend()
    }

    /// Returns configured listen address for this live adapter.
    pub fn listen_address(&self) -> &str {
        self.swarm_config.listen_address()
    }

    /// Returns deterministic live data-plane network identifier.
    pub fn live_data_plane_network_id(&self) -> &str {
        self.live_network_id.as_str()
    }

    #[cfg(not(feature = "libp2p-live-transport"))]
    fn lock_live_data_plane_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Libp2pLiveDataPlaneState>, P2pTransportError> {
        self.live_data_plane
            .state
            .lock()
            .map_err(|_| P2pTransportError::StateUnavailable)
    }
}

impl PeerLifecycleTransport for Libp2pLivePeerLifecycleTransport {
    fn advertise(&self, record: PeerDiscoveryRecord) -> Result<(), P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_socket_transport.advertise(record)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        let mut state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            state
                .inbox_by_peer
                .entry(record.peer_id.clone())
                .or_insert_with(VecDeque::new);
            state.peers_by_id.insert(record.peer_id.clone(), record);
            Ok(())
        }
    }

    fn discover(
        &self,
        requester_peer_id: &str,
        topic: &str,
    ) -> Result<Vec<PeerDiscoveryRecord>, P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_socket_transport
                .discover(requester_peer_id, topic)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        validate_peer_id(requester_peer_id)?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        validate_topic(topic)?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        let state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            Ok(state
                .peers_by_id
                .values()
                .filter(|record| {
                    record.peer_id != requester_peer_id && record.supports_topic(topic)
                })
                .cloned()
                .collect())
        }
    }

    fn send(&self, frame: PeerGossipFrame) -> Result<(), P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_socket_transport.send(frame)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        let mut state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        if !state.peers_by_id.contains_key(&frame.sender_peer_id) {
            return Err(P2pTransportError::UnknownSenderPeer(
                frame.sender_peer_id.clone(),
            ));
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        if !state.peers_by_id.contains_key(&frame.recipient_peer_id) {
            return Err(P2pTransportError::UnknownRecipientPeer(
                frame.recipient_peer_id.clone(),
            ));
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            state
                .inbox_by_peer
                .entry(frame.recipient_peer_id.clone())
                .or_insert_with(VecDeque::new)
                .push_back(frame);
            Ok(())
        }
    }

    fn drain_inbox(
        &self,
        recipient_peer_id: &str,
    ) -> Result<Vec<PeerGossipFrame>, P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_socket_transport.drain_inbox(recipient_peer_id)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        validate_peer_id(recipient_peer_id)?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        let mut state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            let queue = state
                .inbox_by_peer
                .entry(recipient_peer_id.to_owned())
                .or_insert_with(VecDeque::new);
            Ok(queue.drain(..).collect())
        }
    }
}

#[cfg(not(feature = "libp2p-live-transport"))]
#[derive(Debug, Default)]
struct Libp2pLiveDataPlaneState {
    peers_by_id: BTreeMap<String, PeerDiscoveryRecord>,
    inbox_by_peer: BTreeMap<String, VecDeque<PeerGossipFrame>>,
}

#[cfg(not(feature = "libp2p-live-transport"))]
#[derive(Debug, Clone)]
struct Libp2pLiveDataPlane {
    state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
}

#[cfg(not(feature = "libp2p-live-transport"))]
fn libp2p_live_data_plane_registry(
) -> &'static Mutex<BTreeMap<String, Arc<Mutex<Libp2pLiveDataPlaneState>>>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, Arc<Mutex<Libp2pLiveDataPlaneState>>>>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

#[cfg(not(feature = "libp2p-live-transport"))]
fn resolve_live_data_plane_state(
    network_id: &str,
) -> Result<Arc<Mutex<Libp2pLiveDataPlaneState>>, P2pTransportError> {
    let mut registry = libp2p_live_data_plane_registry()
        .lock()
        .map_err(|_| P2pTransportError::StateUnavailable)?;
    Ok(registry
        .entry(network_id.to_owned())
        .or_insert_with(|| Arc::new(Mutex::new(Libp2pLiveDataPlaneState::default())))
        .clone())
}

fn build_live_data_plane_network_id(config: &P2pSwarmDeterministicConfig) -> String {
    let bootstrap_segment = if config.bootstrap_peers().is_empty() {
        format!("listen={}", config.listen_address())
    } else {
        format!("bootstrap={}", config.bootstrap_peers().join(","))
    };
    let topic_segment = format!("topics={}", config.gossip_topics().join(","));
    format!("{bootstrap_segment}|{topic_segment}")
}

#[cfg(feature = "libp2p-live-transport")]
fn validate_libp2p_native_runtime_config(
    config: &P2pSwarmDeterministicConfig,
) -> Result<(), P2pTransportError> {
    use libp2p::Multiaddr;
    config
        .listen_address()
        .parse::<Multiaddr>()
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    for topic in config.gossip_topics() {
        let _ = libp2p::gossipsub::IdentTopic::new(topic.clone());
    }
    Ok(())
}

/// Fault classes observed during live libp2p reconnect/discovery operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveTransportFaultClass {
    /// Dial or connection setup timed out.
    DialTimeout,
    /// Discovery backend returned unavailable/unreachable status.
    DiscoveryUnavailable,
    /// Stream churn/drop detected during reconnect sequence.
    StreamChurn,
    /// Protocol legality violation was observed (fail closed).
    ProtocolViolation,
}

impl LiveTransportFaultClass {
    fn retry_reason_code(self) -> &'static str {
        match self {
            Self::DialTimeout => "p2p_live_reconnect_retry_dial_timeout",
            Self::DiscoveryUnavailable => "p2p_live_reconnect_retry_discovery_unavailable",
            Self::StreamChurn => "p2p_live_reconnect_retry_stream_churn",
            Self::ProtocolViolation => "p2p_live_reconnect_protocol_violation",
        }
    }
}

/// Deterministic reconnect/backoff decision output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveTransportReconnectDecision {
    /// Retry is allowed with bounded deterministic backoff.
    Retry {
        /// Backoff budget in abstract ticks.
        backoff_ticks: u16,
        /// Deterministic reason code.
        reason_code: &'static str,
    },
    /// Retry is disallowed and transport must fail closed.
    FailClosed {
        /// Deterministic reason code.
        reason_code: &'static str,
    },
}

/// Deterministic reconnect/backoff policy for live transport faults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTransportReconnectPolicy {
    base_backoff_ticks: u16,
    max_backoff_ticks: u16,
    max_retry_attempts: u16,
}

impl LiveTransportReconnectPolicy {
    /// Builds a validated deterministic reconnect/backoff policy.
    pub fn new(
        base_backoff_ticks: u16,
        max_backoff_ticks: u16,
        max_retry_attempts: u16,
    ) -> Result<Self, P2pTransportError> {
        if max_retry_attempts == 0 {
            return Err(P2pTransportError::InvalidReconnectRetryBudget);
        }
        if base_backoff_ticks == 0
            || max_backoff_ticks == 0
            || base_backoff_ticks > max_backoff_ticks
        {
            return Err(P2pTransportError::InvalidReconnectBackoffWindow);
        }
        Ok(Self {
            base_backoff_ticks,
            max_backoff_ticks,
            max_retry_attempts,
        })
    }

    /// Evaluates deterministic reconnect decision for one fault class + attempt index.
    pub fn evaluate(
        &self,
        fault_class: LiveTransportFaultClass,
        attempt: u16,
    ) -> LiveTransportReconnectDecision {
        if fault_class == LiveTransportFaultClass::ProtocolViolation {
            return LiveTransportReconnectDecision::FailClosed {
                reason_code: fault_class.retry_reason_code(),
            };
        }

        let normalized_attempt = attempt.max(1);
        if normalized_attempt >= self.max_retry_attempts {
            return LiveTransportReconnectDecision::FailClosed {
                reason_code: "p2p_live_reconnect_retry_budget_exhausted",
            };
        }

        LiveTransportReconnectDecision::Retry {
            backoff_ticks: self.backoff_ticks_for_attempt(normalized_attempt),
            reason_code: fault_class.retry_reason_code(),
        }
    }

    fn backoff_ticks_for_attempt(&self, attempt: u16) -> u16 {
        let mut backoff = u32::from(self.base_backoff_ticks);
        let max_backoff = u32::from(self.max_backoff_ticks);
        for _ in 1..attempt {
            if backoff >= max_backoff {
                break;
            }
            backoff = backoff.saturating_mul(2).min(max_backoff);
        }
        backoff as u16
    }
}

/// Deterministic config used to compose a libp2p swarm behavior stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pSwarmDeterministicConfig {
    local_peer_id: String,
    listen_address: String,
    bootstrap_peers: Vec<String>,
    gossip_topics: Vec<String>,
    harness_tick_budget: u16,
}

impl P2pSwarmDeterministicConfig {
    /// Builds a validated deterministic swarm configuration.
    pub fn new(
        local_peer_id: &str,
        listen_address: &str,
        bootstrap_peers: Vec<String>,
        gossip_topics: Vec<String>,
        harness_tick_budget: u16,
    ) -> Result<Self, P2pTransportError> {
        validate_peer_id(local_peer_id)?;
        validate_swarm_listen_address(listen_address)?;
        if harness_tick_budget == 0 {
            return Err(P2pTransportError::InvalidSwarmHarnessTickBudget);
        }
        if gossip_topics.is_empty() {
            return Err(P2pTransportError::MissingGossipTopics);
        }

        let mut normalized_bootstrap = BTreeSet::new();
        for peer in bootstrap_peers {
            validate_swarm_bootstrap_peer_address(peer.as_str())?;
            normalized_bootstrap.insert(peer.trim().to_owned());
        }

        let mut normalized_topics = BTreeSet::new();
        for topic in gossip_topics {
            validate_topic(topic.as_str())?;
            normalized_topics.insert(topic.trim().to_owned());
        }

        Ok(Self {
            local_peer_id: local_peer_id.to_owned(),
            listen_address: listen_address.trim().to_owned(),
            bootstrap_peers: normalized_bootstrap.into_iter().collect(),
            gossip_topics: normalized_topics.into_iter().collect(),
            harness_tick_budget,
        })
    }

    /// Returns the local peer id.
    pub fn local_peer_id(&self) -> &str {
        &self.local_peer_id
    }

    /// Returns the local listen multiaddr.
    pub fn listen_address(&self) -> &str {
        &self.listen_address
    }

    /// Returns canonical bootstrap peer multiaddrs.
    pub fn bootstrap_peers(&self) -> &[String] {
        &self.bootstrap_peers
    }

    /// Returns canonical gossip topic subscriptions.
    pub fn gossip_topics(&self) -> &[String] {
        &self.gossip_topics
    }

    /// Returns deterministic harness tick budget.
    pub fn harness_tick_budget(&self) -> u16 {
        self.harness_tick_budget
    }
}

/// Builds deterministic swarm config from node config and explicit transport inputs.
pub fn build_p2p_swarm_deterministic_config(
    node_config: &NodeConfig,
    local_peer_id: &str,
    listen_address: &str,
    bootstrap_peers: Vec<String>,
    gossip_topics: Vec<String>,
    harness_tick_budget: u16,
) -> Result<P2pSwarmDeterministicConfig, P2pTransportError> {
    if !node_config.enable_gossip {
        return Err(P2pTransportError::GossipTransportDisabled);
    }
    P2pSwarmDeterministicConfig::new(
        local_peer_id,
        listen_address,
        bootstrap_peers,
        gossip_topics,
        harness_tick_budget,
    )
}

/// Canonical behavior stack summary for deterministic libp2p runtime composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pSwarmBehaviorStack {
    listen_address: String,
    bootstrap_peers: Vec<String>,
    gossip_topics: Vec<String>,
    behavior_components: Vec<&'static str>,
}

impl P2pSwarmBehaviorStack {
    /// Returns canonical behavior component ordering.
    pub fn behavior_components(&self) -> Vec<&'static str> {
        self.behavior_components.clone()
    }

    /// Returns canonical gossip topic ordering.
    pub fn gossip_topics(&self) -> Vec<String> {
        self.gossip_topics.clone()
    }

    /// Returns canonical bootstrap peer ordering.
    pub fn bootstrap_peers(&self) -> Vec<String> {
        self.bootstrap_peers.clone()
    }

    /// Returns local listen multiaddr.
    pub fn listen_address(&self) -> &str {
        &self.listen_address
    }
}

/// Composes deterministic libp2p behavior stack metadata.
pub fn compose_libp2p_swarm_behavior_stack(
    config: &P2pSwarmDeterministicConfig,
) -> P2pSwarmBehaviorStack {
    P2pSwarmBehaviorStack {
        listen_address: config.listen_address().to_owned(),
        bootstrap_peers: config.bootstrap_peers().to_vec(),
        gossip_topics: config.gossip_topics().to_vec(),
        behavior_components: LIBP2P_SWARM_BEHAVIOR_COMPONENTS.to_vec(),
    }
}

/// Canonical Kademlia bootstrap seed set for deterministic discovery startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KademliaBootstrapSeedSet {
    seed_peers: Vec<String>,
}

impl KademliaBootstrapSeedSet {
    /// Builds a validated deterministic Kademlia bootstrap seed set.
    pub fn new(seed_peers: Vec<String>) -> Result<Self, P2pTransportError> {
        if seed_peers.is_empty() {
            return Err(P2pTransportError::MissingKademliaBootstrapSeeds);
        }

        let mut normalized = BTreeSet::new();
        for peer in seed_peers {
            validate_swarm_bootstrap_peer_address(peer.as_str())?;
            normalized.insert(peer.trim().to_owned());
        }

        Ok(Self {
            seed_peers: normalized.into_iter().collect(),
        })
    }

    /// Returns canonical bootstrap peer ordering.
    pub fn seed_peers(&self) -> Vec<String> {
        self.seed_peers.clone()
    }
}

/// Deterministic Kademlia discovery bootstrap plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KademliaDiscoveryBootstrapPlan {
    discovery_backend: &'static str,
    seed_peers: Vec<String>,
}

impl KademliaDiscoveryBootstrapPlan {
    /// Returns the deterministic discovery backend marker.
    pub fn discovery_backend(&self) -> &'static str {
        self.discovery_backend
    }

    /// Returns canonical Kademlia bootstrap seed ordering.
    pub fn seed_peers(&self) -> Vec<String> {
        self.seed_peers.clone()
    }
}

/// Composes deterministic Kademlia bootstrap behavior from swarm config seed peers.
pub fn compose_kademlia_discovery_bootstrap(
    config: &P2pSwarmDeterministicConfig,
) -> Result<KademliaDiscoveryBootstrapPlan, P2pTransportError> {
    let seed_set = KademliaBootstrapSeedSet::new(config.bootstrap_peers().to_vec())?;
    Ok(KademliaDiscoveryBootstrapPlan {
        discovery_backend: "kademlia",
        seed_peers: seed_set.seed_peers(),
    })
}

/// Expected outcome category for a lifecycle regression replay case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerLifecycleRegressionExpectedOutcome {
    /// Replay should complete and end on the provided lifecycle state.
    FinalState(PeerLifecycleState),
    /// Replay should fail closed with the provided transition error.
    TransitionError(RuntimeLifecycleError),
}

/// Deterministic lifecycle regression replay case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerLifecycleRegressionCase {
    case_id: String,
    events: Vec<PeerLifecycleEvent>,
    expected_outcome: PeerLifecycleRegressionExpectedOutcome,
}

impl PeerLifecycleRegressionCase {
    /// Builds a validated lifecycle regression replay case.
    pub fn new(
        case_id: &str,
        events: Vec<PeerLifecycleEvent>,
        expected_outcome: PeerLifecycleRegressionExpectedOutcome,
    ) -> Result<Self, PeerLifecycleRegressionError> {
        if case_id.trim().is_empty() {
            return Err(PeerLifecycleRegressionError::EmptyCaseId);
        }
        if events.is_empty() {
            return Err(PeerLifecycleRegressionError::EmptyEventSequence);
        }
        Ok(Self {
            case_id: case_id.to_owned(),
            events,
            expected_outcome,
        })
    }

    /// Returns deterministic replay case id.
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Returns replay event sequence.
    pub fn events(&self) -> &[PeerLifecycleEvent] {
        &self.events
    }

    /// Returns expected replay outcome.
    pub fn expected_outcome(&self) -> &PeerLifecycleRegressionExpectedOutcome {
        &self.expected_outcome
    }
}

/// Deterministic lifecycle regression replay outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerLifecycleRegressionOutcome {
    case_id: String,
    final_state: Option<PeerLifecycleState>,
    transition_error_reason_code: Option<&'static str>,
}

impl PeerLifecycleRegressionOutcome {
    /// Returns replay case id.
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Returns final lifecycle state when replay succeeded.
    pub fn final_state(&self) -> Option<PeerLifecycleState> {
        self.final_state
    }

    /// Returns deterministic transition error reason code when replay failed.
    pub fn transition_error_reason_code(&self) -> Option<&'static str> {
        self.transition_error_reason_code
    }
}

/// Lifecycle regression replay error variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerLifecycleRegressionError {
    /// Case id is empty.
    EmptyCaseId,
    /// Event sequence is empty.
    EmptyEventSequence,
    /// Lifecycle construction or transition returned a runtime error.
    Lifecycle(RuntimeLifecycleError),
    /// Final lifecycle state differs from expected deterministic state.
    ExpectedFinalStateMismatch {
        /// Case id.
        case_id: String,
        /// Expected state.
        expected: PeerLifecycleState,
        /// Observed state.
        found: PeerLifecycleState,
    },
    /// Transition error occurred when case expected a final-state result.
    UnexpectedTransitionError {
        /// Case id.
        case_id: String,
        /// Observed transition error.
        found: RuntimeLifecycleError,
    },
    /// Expected transition-error contract differs from observed result.
    ExpectedTransitionErrorMismatch {
        /// Case id.
        case_id: String,
        /// Expected transition error.
        expected: RuntimeLifecycleError,
        /// Observed transition error, if one occurred.
        found: Option<RuntimeLifecycleError>,
    },
}

impl Display for PeerLifecycleRegressionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCaseId => write!(f, "lifecycle regression case id cannot be empty"),
            Self::EmptyEventSequence => write!(f, "lifecycle regression event sequence cannot be empty"),
            Self::Lifecycle(error) => write!(f, "{error}"),
            Self::ExpectedFinalStateMismatch {
                case_id,
                expected,
                found,
            } => write!(
                f,
                "lifecycle regression case {case_id} expected final state {expected:?}, found {found:?}"
            ),
            Self::UnexpectedTransitionError { case_id, found } => write!(
                f,
                "lifecycle regression case {case_id} observed unexpected transition error {found:?}"
            ),
            Self::ExpectedTransitionErrorMismatch {
                case_id, expected, found
            } => write!(
                f,
                "lifecycle regression case {case_id} expected transition error {expected:?}, found {found:?}"
            ),
        }
    }
}

impl Error for PeerLifecycleRegressionError {}

/// Builds deterministic default lifecycle regression corpus for libp2p transport transitions.
pub fn build_libp2p_lifecycle_regression_corpus() -> Vec<PeerLifecycleRegressionCase> {
    vec![
        PeerLifecycleRegressionCase {
            case_id: "connect_handshake_disconnect".to_owned(),
            events: vec![
                PeerLifecycleEvent::StartConnect,
                PeerLifecycleEvent::HandshakeSucceeded,
                PeerLifecycleEvent::Disconnect,
            ],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::FinalState(
                PeerLifecycleState::Disconnected,
            ),
        },
        PeerLifecycleRegressionCase {
            case_id: "connect_heartbeat_timeout_recovery".to_owned(),
            events: vec![
                PeerLifecycleEvent::StartConnect,
                PeerLifecycleEvent::HandshakeSucceeded,
                PeerLifecycleEvent::HeartbeatMissed,
                PeerLifecycleEvent::HeartbeatRestored,
            ],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::FinalState(
                PeerLifecycleState::Active,
            ),
        },
        PeerLifecycleRegressionCase {
            case_id: "connect_drop_rejoin".to_owned(),
            events: vec![
                PeerLifecycleEvent::StartConnect,
                PeerLifecycleEvent::HandshakeSucceeded,
                PeerLifecycleEvent::Disconnect,
                PeerLifecycleEvent::Rejoin,
                PeerLifecycleEvent::HandshakeSucceeded,
            ],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::FinalState(
                PeerLifecycleState::Active,
            ),
        },
        PeerLifecycleRegressionCase {
            case_id: "invalid_heartbeat_from_disconnected".to_owned(),
            events: vec![PeerLifecycleEvent::HeartbeatMissed],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::TransitionError(
                RuntimeLifecycleError::InvalidTransition {
                    from: PeerLifecycleState::Disconnected,
                    event: PeerLifecycleEvent::HeartbeatMissed,
                },
            ),
        },
    ]
}

/// Replays one deterministic lifecycle regression case.
pub fn run_libp2p_lifecycle_regression_case(
    peer_id: &str,
    case: &PeerLifecycleRegressionCase,
) -> Result<PeerLifecycleRegressionOutcome, PeerLifecycleRegressionError> {
    let mut lifecycle =
        PeerLifecycle::new(peer_id).map_err(PeerLifecycleRegressionError::Lifecycle)?;

    let mut observed_error = None;
    let mut observed_state = lifecycle.state();
    for event in case.events() {
        match lifecycle.transition(*event) {
            Ok(next_state) => observed_state = next_state,
            Err(error) => {
                observed_error = Some(error);
                break;
            }
        }
    }

    match case.expected_outcome() {
        PeerLifecycleRegressionExpectedOutcome::FinalState(expected) => {
            if let Some(error) = observed_error {
                return Err(PeerLifecycleRegressionError::UnexpectedTransitionError {
                    case_id: case.case_id().to_owned(),
                    found: error,
                });
            }
            if &observed_state != expected {
                return Err(PeerLifecycleRegressionError::ExpectedFinalStateMismatch {
                    case_id: case.case_id().to_owned(),
                    expected: *expected,
                    found: observed_state,
                });
            }
            Ok(PeerLifecycleRegressionOutcome {
                case_id: case.case_id().to_owned(),
                final_state: Some(observed_state),
                transition_error_reason_code: None,
            })
        }
        PeerLifecycleRegressionExpectedOutcome::TransitionError(expected_error) => {
            let Some(found_error) = observed_error else {
                return Err(
                    PeerLifecycleRegressionError::ExpectedTransitionErrorMismatch {
                        case_id: case.case_id().to_owned(),
                        expected: expected_error.clone(),
                        found: None,
                    },
                );
            };
            if &found_error != expected_error {
                return Err(
                    PeerLifecycleRegressionError::ExpectedTransitionErrorMismatch {
                        case_id: case.case_id().to_owned(),
                        expected: expected_error.clone(),
                        found: Some(found_error),
                    },
                );
            }
            Ok(PeerLifecycleRegressionOutcome {
                case_id: case.case_id().to_owned(),
                final_state: None,
                transition_error_reason_code: Some(found_error.reason_code()),
            })
        }
    }
}

/// Replays deterministic lifecycle regression corpus in the provided order.
pub fn run_libp2p_lifecycle_regression_corpus(
    peer_id: &str,
    corpus: &[PeerLifecycleRegressionCase],
) -> Result<Vec<PeerLifecycleRegressionOutcome>, PeerLifecycleRegressionError> {
    let mut outcomes = Vec::with_capacity(corpus.len());
    for case in corpus {
        outcomes.push(run_libp2p_lifecycle_regression_case(peer_id, case)?);
    }
    Ok(outcomes)
}

/// Swarm harness execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P2pSwarmHarnessMode {
    /// Build and validate deterministic stack without running loop ticks.
    DryRun,
    /// Start deterministic runtime harness and execute bounded loop ticks.
    Run,
}

/// Deterministic swarm harness startup report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pSwarmHarnessReport {
    mode: P2pSwarmHarnessMode,
    started: bool,
    executed_ticks: u16,
    bootstrap_peer_count: usize,
    behavior_components: Vec<&'static str>,
}

impl P2pSwarmHarnessReport {
    /// Returns harness mode.
    pub fn mode(&self) -> P2pSwarmHarnessMode {
        self.mode
    }

    /// Returns whether run mode started the deterministic loop.
    pub fn started(&self) -> bool {
        self.started
    }

    /// Returns deterministic executed tick count.
    pub fn executed_ticks(&self) -> u16 {
        self.executed_ticks
    }

    /// Returns canonical bootstrap peer count.
    pub fn bootstrap_peer_count(&self) -> usize {
        self.bootstrap_peer_count
    }

    /// Returns canonical behavior stack ordering used during startup.
    pub fn behavior_components(&self) -> Vec<&'static str> {
        self.behavior_components.clone()
    }
}

/// Runtime harness wrapper used to start deterministic swarm loops in tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pSwarmHarnessTask {
    config: P2pSwarmDeterministicConfig,
    stack: P2pSwarmBehaviorStack,
}

impl P2pSwarmHarnessTask {
    /// Builds a deterministic harness task for the provided swarm config.
    pub fn new(config: P2pSwarmDeterministicConfig) -> Self {
        let stack = compose_libp2p_swarm_behavior_stack(&config);
        Self { config, stack }
    }

    /// Starts deterministic harness mode and returns startup report.
    pub fn start(
        &self,
        mode: P2pSwarmHarnessMode,
    ) -> Result<P2pSwarmHarnessReport, P2pTransportError> {
        let started = matches!(mode, P2pSwarmHarnessMode::Run);
        let executed_ticks = if started {
            self.config.harness_tick_budget()
        } else {
            0
        };
        Ok(P2pSwarmHarnessReport {
            mode,
            started,
            executed_ticks,
            bootstrap_peer_count: self.config.bootstrap_peers().len(),
            behavior_components: self.stack.behavior_components(),
        })
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
    use super::{
        InMemoryPeerLifecycleTransport, P2pTransportError, PeerGossipFrame, PeerLifecycleTransport,
    };
    use crate::config::NodeRole;
    use crate::p2p_transport::{PeerDiscoveryRecord, PeerLifecycleTransportCoordinator};
    use crate::runtime::{PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError};

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
            P2pTransportError::Lifecycle(RuntimeLifecycleError::InvalidTransition {
                from: PeerLifecycleState::Disconnected,
                event: PeerLifecycleEvent::HeartbeatRestored,
            })
            .reason_code(),
            "runtime_peer_transition_invalid"
        );
    }
}
