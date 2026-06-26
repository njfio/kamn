use super::BridgePlatform;

/// Raw inbound message payload received from a bridge connector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeInboundEnvelope {
    /// External message id carried by this public contract model.
    pub external_message_id: String,
    /// External sender id carried by this public contract model.
    pub external_sender_id: String,
    /// External channel id carried by this public contract model.
    pub external_channel_id: String,
    /// Target agent did carried by this public contract model.
    pub target_agent_did: String,
    /// Body carried by this public contract model.
    pub body: String,
    /// Received at carried by this public contract model.
    pub received_at: String,
    /// Received at unix carried by this public contract model.
    pub received_at_unix: u64,
}

/// Canonical inbound shape produced by a [`BridgeAdapter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedInboundMessage {
    /// Bridge message id carried by this public contract model.
    pub bridge_message_id: String,
    /// Sender handle carried by this public contract model.
    pub sender_handle: String,
    /// Source channel carried by this public contract model.
    pub source_channel: String,
    /// Target agent did carried by this public contract model.
    pub target_agent_did: String,
    /// Body carried by this public contract model.
    pub body: String,
    /// Received at carried by this public contract model.
    pub received_at: String,
    /// Received at unix carried by this public contract model.
    pub received_at_unix: u64,
    /// Platform carried by this public contract model.
    pub platform: BridgePlatform,
}

/// Outbound request submitted for bridge delivery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeOutboundRequest {
    /// Request id carried by this public contract model.
    pub request_id: String,
    /// From agent did carried by this public contract model.
    pub from_agent_did: String,
    /// Destination channel id carried by this public contract model.
    pub destination_channel_id: String,
    /// Body carried by this public contract model.
    pub body: String,
    /// Created at carried by this public contract model.
    pub created_at: String,
}

/// Platform-ready outbound envelope emitted by a [`BridgeAdapter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeOutboundEnvelope {
    /// Request id carried by this public contract model.
    pub request_id: String,
    /// Destination channel id carried by this public contract model.
    pub destination_channel_id: String,
    /// Payload carried by this public contract model.
    pub payload: String,
    /// Platform carried by this public contract model.
    pub platform: BridgePlatform,
}
