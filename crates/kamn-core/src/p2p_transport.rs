//! Deterministic peer discovery and gossip transport adapters for runtime integration.

use crate::config::NodeRole;
use crate::runtime::{
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeTransportProfile,
};
#[cfg(feature = "libp2p-live-transport")]
use libp2p::{gossipsub, identify, noise, swarm::Swarm, tcp, yamux, Multiaddr, SwarmBuilder};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

mod adapter;
mod error;
mod lifecycle_regression;
#[cfg(feature = "libp2p-live-transport")]
mod native_runtime;
mod runtime_event;
mod swarm_stack;
mod validation;
pub use adapter::{
    InMemoryPeerLifecycleTransport, PeerDiscoveryRecord, PeerGossipFrame, PeerLifecycleTransport,
    UdpPeerLifecycleTransport,
};
pub use error::{Libp2pRuntimeAdapterOperation, P2pTransportError};
pub use lifecycle_regression::{
    build_libp2p_lifecycle_regression_corpus, run_libp2p_lifecycle_regression_case,
    run_libp2p_lifecycle_regression_corpus, PeerLifecycleRegressionCase,
    PeerLifecycleRegressionError, PeerLifecycleRegressionExpectedOutcome,
    PeerLifecycleRegressionOutcome,
};
#[cfg(feature = "libp2p-live-transport")]
use native_runtime::Libp2pNativeRuntimeAdapterLoop;
pub use runtime_event::{Libp2pBehaviorFailureClass, Libp2pRuntimeEvent, Libp2pRuntimeEventKind};
pub use swarm_stack::{
    build_p2p_swarm_deterministic_config, compose_kademlia_discovery_bootstrap,
    compose_libp2p_swarm_behavior_stack, KademliaBootstrapSeedSet, KademliaDiscoveryBootstrapPlan,
    LiveTransportFaultClass, LiveTransportReconnectDecision, LiveTransportReconnectPolicy,
    P2pSwarmBehaviorStack, P2pSwarmDeterministicConfig, P2pSwarmHarnessMode, P2pSwarmHarnessReport,
    P2pSwarmHarnessTask,
};
use validation::{validate_peer_id, validate_topic};

const LIBP2P_SWARM_BEHAVIOR_COMPONENTS: [&str; 6] =
    ["tcp", "noise", "yamux", "identify", "kad", "gossipsub"];
const LIBP2P_IDENTIFY_PROTOCOL_ID: &str = "/kamn/libp2p-live/1.0.0";
const LIBP2P_TOPIC_NAMESPACE: &str = "kamn/v1/";

#[cfg(feature = "libp2p-live-transport")]
#[derive(libp2p::swarm::NetworkBehaviour)]
struct Libp2pDeterministicRuntimeBehaviour {
    gossipsub: gossipsub::Behaviour,
    identify: identify::Behaviour,
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
            Self::NativeSocket => "native-libp2p-swarm",
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
    native_runtime_loop: Libp2pNativeRuntimeAdapterLoop,
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
        let state = resolve_live_data_plane_state(network_id.as_str())?;
        #[cfg(feature = "libp2p-live-transport")]
        validate_libp2p_native_runtime_config(&config)?;
        #[cfg(feature = "libp2p-live-transport")]
        let native_runtime_loop =
            Libp2pNativeRuntimeAdapterLoop::start(config.clone(), state.clone())?;
        Ok(Self {
            swarm_config: config,
            harness_report,
            live_network_id: network_id.clone(),
            #[cfg(feature = "libp2p-live-transport")]
            native_runtime_loop,
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

    /// Drains normalized runtime events emitted by this transport adapter.
    pub fn drain_runtime_events(&self) -> Result<Vec<Libp2pRuntimeEvent>, P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_runtime_loop.drain_runtime_events()
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            let mut state = self.lock_live_data_plane_state()?;
            Ok(state.runtime_events.drain(..).collect())
        }
    }

    #[cfg(feature = "libp2p-live-transport")]
    /// Returns deterministic native runtime loop marker for feature-enabled adapter wiring.
    pub fn native_runtime_loop_marker(&self) -> &'static str {
        self.native_runtime_loop.marker()
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
            self.native_runtime_loop.advertise(record)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        let mut state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            let event = Libp2pRuntimeEvent::peer_advertised(record.peer_id.as_str())?;
            state
                .inbox_by_peer
                .entry(record.peer_id.clone())
                .or_insert_with(VecDeque::new);
            state.peers_by_id.insert(record.peer_id.clone(), record);
            state.runtime_events.push_back(event);
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
            self.native_runtime_loop.discover(requester_peer_id, topic)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        validate_peer_id(requester_peer_id)?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        validate_topic(topic)?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        let mut state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            let discovered = state
                .peers_by_id
                .values()
                .filter(|record| {
                    record.peer_id != requester_peer_id && record.supports_topic(topic)
                })
                .cloned()
                .collect::<Vec<PeerDiscoveryRecord>>();
            for record in &discovered {
                state
                    .runtime_events
                    .push_back(Libp2pRuntimeEvent::peer_discovered(
                        record.peer_id.as_str(),
                        topic,
                    )?);
            }
            Ok(discovered)
        }
    }

    fn send(&self, frame: PeerGossipFrame) -> Result<(), P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_runtime_loop.send(frame)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        let mut state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        if !state.peers_by_id.contains_key(&frame.sender_peer_id) {
            state
                .runtime_events
                .push_back(Libp2pRuntimeEvent::behavior_failure(
                    Libp2pBehaviorFailureClass::UnknownSenderPeer,
                    Some(frame.sender_peer_id.as_str()),
                    Some(frame.topic.as_str()),
                )?);
            return Err(P2pTransportError::UnknownSenderPeer(
                frame.sender_peer_id.clone(),
            ));
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        if !state.peers_by_id.contains_key(&frame.recipient_peer_id) {
            state
                .runtime_events
                .push_back(Libp2pRuntimeEvent::behavior_failure(
                    Libp2pBehaviorFailureClass::UnknownRecipientPeer,
                    Some(frame.recipient_peer_id.as_str()),
                    Some(frame.topic.as_str()),
                )?);
            return Err(P2pTransportError::UnknownRecipientPeer(
                frame.recipient_peer_id.clone(),
            ));
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            let published = Libp2pRuntimeEvent::gossip_published(
                frame.sender_peer_id.as_str(),
                frame.topic.as_str(),
                frame.payload.as_str(),
            )?;
            let received = Libp2pRuntimeEvent::gossip_received(
                frame.recipient_peer_id.as_str(),
                frame.topic.as_str(),
                frame.payload.as_str(),
            )?;
            state
                .inbox_by_peer
                .entry(frame.recipient_peer_id.clone())
                .or_insert_with(VecDeque::new)
                .push_back(frame);
            state.runtime_events.push_back(published);
            state.runtime_events.push_back(received);
            Ok(())
        }
    }

    fn drain_inbox(
        &self,
        recipient_peer_id: &str,
    ) -> Result<Vec<PeerGossipFrame>, P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_runtime_loop.drain_inbox(recipient_peer_id)
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

#[derive(Debug, Default)]
struct Libp2pLiveDataPlaneState {
    peers_by_id: BTreeMap<String, PeerDiscoveryRecord>,
    inbox_by_peer: BTreeMap<String, VecDeque<PeerGossipFrame>>,
    runtime_events: VecDeque<Libp2pRuntimeEvent>,
}

#[cfg(not(feature = "libp2p-live-transport"))]
#[derive(Debug, Clone)]
struct Libp2pLiveDataPlane {
    state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
}

fn libp2p_live_data_plane_registry(
) -> &'static Mutex<BTreeMap<String, Arc<Mutex<Libp2pLiveDataPlaneState>>>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, Arc<Mutex<Libp2pLiveDataPlaneState>>>>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

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

/// Returns canonical identify protocol id for deterministic libp2p runtime composition.
pub fn canonical_libp2p_identify_protocol_id() -> &'static str {
    LIBP2P_IDENTIFY_PROTOCOL_ID
}

/// Returns canonical gossipsub topic id for deterministic runtime policy checks.
pub fn canonical_libp2p_topic_id(topic: &str) -> Result<String, P2pTransportError> {
    validate_topic(topic)?;
    Ok(format!("{LIBP2P_TOPIC_NAMESPACE}{}", topic.trim()))
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
    for bootstrap_peer in config.bootstrap_peers() {
        bootstrap_peer
            .parse::<Multiaddr>()
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    }
    for topic in config.gossip_topics() {
        let topic_id = canonical_libp2p_topic_id(topic.as_str())
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
        let _ = libp2p::gossipsub::IdentTopic::new(topic_id);
    }
    Ok(())
}

#[cfg(feature = "libp2p-live-transport")]
fn build_libp2p_runtime_swarm(
    config: &P2pSwarmDeterministicConfig,
) -> Result<Swarm<Libp2pDeterministicRuntimeBehaviour>, P2pTransportError> {
    let swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?
        .with_behaviour(|key| {
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .validation_mode(gossipsub::ValidationMode::Permissive)
                .build()
                .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
            let mut gossipsub_behavior = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
            for topic in config.gossip_topics() {
                let topic_id = canonical_libp2p_topic_id(topic.as_str())
                    .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
                gossipsub_behavior
                    .subscribe(&gossipsub::IdentTopic::new(topic_id))
                    .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
            }
            Ok(Libp2pDeterministicRuntimeBehaviour {
                gossipsub: gossipsub_behavior,
                identify: identify::Behaviour::new(identify::Config::new(
                    canonical_libp2p_identify_protocol_id().to_owned(),
                    key.public(),
                )),
            })
        })
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?
        .build();
    Ok(swarm)
}

#[cfg(feature = "libp2p-live-transport")]
fn apply_libp2p_runtime_network_config(
    swarm: &mut Swarm<Libp2pDeterministicRuntimeBehaviour>,
    config: &P2pSwarmDeterministicConfig,
) -> Result<(), P2pTransportError> {
    let listen_multiaddr = config
        .listen_address()
        .parse::<Multiaddr>()
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    Swarm::listen_on(swarm, listen_multiaddr)
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;

    for bootstrap_peer in config.bootstrap_peers() {
        let bootstrap_multiaddr = bootstrap_peer
            .parse::<Multiaddr>()
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
        let _ = Swarm::dial(swarm, bootstrap_multiaddr);
    }
    Ok(())
}

#[cfg(feature = "libp2p-live-transport")]
fn validate_libp2p_runtime_stack_composition(
    config: &P2pSwarmDeterministicConfig,
) -> Result<(), P2pTransportError> {
    let config = config.clone();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;

    runtime.block_on(async move {
        let mut swarm = build_libp2p_runtime_swarm(&config)?;
        apply_libp2p_runtime_network_config(&mut swarm, &config)?;
        Ok(())
    })
}

#[cfg(feature = "libp2p-live-transport")]
fn runtime_channel_closed_behavior_failure_class(
    operation: Libp2pRuntimeAdapterOperation,
) -> Libp2pBehaviorFailureClass {
    match operation {
        Libp2pRuntimeAdapterOperation::Connect => {
            Libp2pBehaviorFailureClass::RuntimeConnectChannelClosed
        }
        Libp2pRuntimeAdapterOperation::Discover => {
            Libp2pBehaviorFailureClass::RuntimeDiscoverChannelClosed
        }
        Libp2pRuntimeAdapterOperation::Publish => {
            Libp2pBehaviorFailureClass::RuntimePublishChannelClosed
        }
        Libp2pRuntimeAdapterOperation::Receive => {
            Libp2pBehaviorFailureClass::RuntimeReceiveChannelClosed
        }
        Libp2pRuntimeAdapterOperation::EventDrain => {
            Libp2pBehaviorFailureClass::RuntimeEventDrainChannelClosed
        }
    }
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
        (
            super::Libp2pNativeRuntimeAdapterLoop::build_closed_for_test(state.clone()),
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
