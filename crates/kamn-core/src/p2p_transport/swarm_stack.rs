use crate::config::NodeConfig;
use std::collections::BTreeSet;

use super::validation::{
    validate_peer_id, validate_swarm_bootstrap_peer_address, validate_swarm_listen_address,
    validate_topic,
};
use super::{
    canonical_libp2p_identify_protocol_id, P2pTransportError, LIBP2P_SWARM_BEHAVIOR_COMPONENTS,
    LIBP2P_TOPIC_NAMESPACE,
};

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
    identify_protocol_id: &'static str,
    gossip_topic_namespace: &'static str,
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

    /// Returns canonical identify protocol id.
    pub fn identify_protocol_id(&self) -> &'static str {
        self.identify_protocol_id
    }

    /// Returns canonical topic namespace prefix used during topic normalization.
    pub fn gossip_topic_namespace(&self) -> &'static str {
        self.gossip_topic_namespace
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
        identify_protocol_id: canonical_libp2p_identify_protocol_id(),
        gossip_topic_namespace: LIBP2P_TOPIC_NAMESPACE,
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
        let behavior_components = self.stack.behavior_components();
        #[cfg(feature = "libp2p-live-transport")]
        let mut behavior_components = behavior_components;
        #[cfg(feature = "libp2p-live-transport")]
        if started {
            super::validate_libp2p_runtime_stack_composition(&self.config)?;
            behavior_components.push("libp2p-runtime-swarm");
        }
        Ok(P2pSwarmHarnessReport {
            mode,
            started,
            executed_ticks,
            bootstrap_peer_count: self.config.bootstrap_peers().len(),
            behavior_components,
        })
    }
}
