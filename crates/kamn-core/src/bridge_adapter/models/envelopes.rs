use super::BridgePlatform;

/// Raw inbound message payload received from a bridge connector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeInboundEnvelope {
    pub external_message_id: String,
    pub external_sender_id: String,
    pub external_channel_id: String,
    pub target_agent_did: String,
    pub body: String,
    pub received_at: String,
    pub received_at_unix: u64,
}

/// Canonical inbound shape produced by a [`BridgeAdapter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedInboundMessage {
    pub bridge_message_id: String,
    pub sender_handle: String,
    pub source_channel: String,
    pub target_agent_did: String,
    pub body: String,
    pub received_at: String,
    pub received_at_unix: u64,
    pub platform: BridgePlatform,
}

/// Outbound request submitted for bridge delivery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeOutboundRequest {
    pub request_id: String,
    pub from_agent_did: String,
    pub destination_channel_id: String,
    pub body: String,
    pub created_at: String,
}

/// Platform-ready outbound envelope emitted by a [`BridgeAdapter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeOutboundEnvelope {
    pub request_id: String,
    pub destination_channel_id: String,
    pub payload: String,
    pub platform: BridgePlatform,
}
