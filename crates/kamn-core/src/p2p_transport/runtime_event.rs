use super::{canonical_libp2p_topic_id, validate_peer_id, P2pTransportError};

const LIBP2P_RUNTIME_EVENT_SCHEMA_MARKER: &str = "kamn.libp2p.runtime-event.v1";

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
    /// Runtime dispatch enqueue rejected by deterministic backpressure.
    RuntimeBackpressureRejectNewEnqueue,
    /// Runtime dispatch purged stale disconnected queue by deterministic backpressure.
    RuntimeBackpressurePurgeStalePeerQueue,
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
            Self::RuntimeBackpressureRejectNewEnqueue => "runtime_backpressure_reject_new_enqueue",
            Self::RuntimeBackpressurePurgeStalePeerQueue => {
                "runtime_backpressure_purge_stale_peer_queue"
            }
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
