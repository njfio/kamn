use crate::runtime::{PeerLifecycleState, RuntimeBackpressureError, RuntimeLifecycleError};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Deterministic p2p discovery and gossip transport error variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Libp2pRuntimeAdapterOperation {
    /// Connect/advertise operation over adapter command bridge.
    Connect,
    /// Discover operation over adapter command bridge.
    Discover,
    /// Publish/send operation over adapter command bridge.
    Publish,
    /// Receive/drain inbox operation over adapter command bridge.
    Receive,
    /// Runtime-event drain operation over adapter command bridge.
    EventDrain,
}

impl Libp2pRuntimeAdapterOperation {
    pub(super) fn channel_closed_reason_code(self) -> &'static str {
        match self {
            Self::Connect => "p2p_libp2p_runtime_connect_channel_closed",
            Self::Discover => "p2p_libp2p_runtime_discover_channel_closed",
            Self::Publish => "p2p_libp2p_runtime_publish_channel_closed",
            Self::Receive => "p2p_libp2p_runtime_receive_channel_closed",
            Self::EventDrain => "p2p_libp2p_runtime_event_drain_channel_closed",
        }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Connect => "connect",
            Self::Discover => "discover",
            Self::Publish => "publish",
            Self::Receive => "receive",
            Self::EventDrain => "event-drain",
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
    /// Runtime backpressure input/policy validation error.
    RuntimeBackpressure(RuntimeBackpressureError),
    /// Runtime backpressure rejected enqueue for queue saturation.
    RuntimeBackpressureRejected {
        /// Deterministic reason code for queue saturation rejection.
        reason_code: &'static str,
        /// Queue utilization per mille when rejection occurred.
        queue_utilization_per_mille: u16,
    },
    /// Runtime backpressure purged stale queue while peer was disconnected.
    RuntimeBackpressurePurgedStalePeerQueue {
        /// Deterministic reason code for stale queue purge.
        reason_code: &'static str,
        /// Number of purged queue entries.
        purged_entries: usize,
    },
}

impl From<RuntimeLifecycleError> for P2pTransportError {
    fn from(value: RuntimeLifecycleError) -> Self {
        Self::Lifecycle(value)
    }
}

impl From<RuntimeBackpressureError> for P2pTransportError {
    fn from(value: RuntimeBackpressureError) -> Self {
        Self::RuntimeBackpressure(value)
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
            Self::RuntimeBackpressure(error) => error.reason_code(),
            Self::RuntimeBackpressureRejected { reason_code, .. } => reason_code,
            Self::RuntimeBackpressurePurgedStalePeerQueue { reason_code, .. } => reason_code,
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
            Self::RuntimeBackpressure(error) => write!(f, "{error}"),
            Self::RuntimeBackpressureRejected {
                queue_utilization_per_mille,
                ..
            } => write!(
                f,
                "p2p runtime inbox enqueue rejected by backpressure at {queue_utilization_per_mille} per mille utilization"
            ),
            Self::RuntimeBackpressurePurgedStalePeerQueue { purged_entries, .. } => write!(
                f,
                "p2p runtime inbox stale queue purged by backpressure; purged {purged_entries} entries"
            ),
        }
    }
}

impl Error for P2pTransportError {}
