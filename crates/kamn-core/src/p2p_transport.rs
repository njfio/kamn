//! Deterministic peer discovery and gossip transport adapters for runtime integration.

use crate::config::{NodeConfig, NodeRole};
use crate::runtime::{
    PeerLifecycle, PeerLifecycleEvent, PeerLifecycleState, RuntimeLifecycleError,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};

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
    /// Swarm composition requested while gossip transport is disabled.
    GossipTransportDisabled,
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
            Self::GossipTransportDisabled => {
                write!(
                    f,
                    "p2p swarm composition requires gossip transport to be enabled"
                )
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
